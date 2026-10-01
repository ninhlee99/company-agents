"""
Neural Talking Head Generator (v2 - Ultra High-Def Mouth Inpainting)
- Uses Wav2Lip Neural GAN on Apple Silicon Metal (MPS GPU).
- Mouth-Only Gaussian Alpha Blending: Keeps eyes, glasses, hair, and studio 100% sharp.
- Biological Eye Blinking & Conversational Head Dynamics.
- 3D Camera Parallax Orbit.
"""

import os
import sys
import math
import cv2
import numpy as np
import torch
import librosa

sys.path.append(os.path.abspath("src/engine/wav2lip_repo"))
from models import Wav2Lip

def get_device():
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")

def load_wav2lip_model(checkpoint_path):
    device = get_device()
    model = Wav2Lip()
    print(f"Loading Wav2Lip checkpoint from {checkpoint_path}...")
    checkpoint = torch.load(checkpoint_path, map_location=device)
    s = checkpoint["state_dict"]
    new_s = {}
    for k, v in s.items():
        new_s[k.replace('module.', '')] = v
    model.load_state_dict(new_s)
    model = model.to(device)
    model.eval()
    print("Wav2Lip GAN Model loaded successfully on", device)
    return model, device

def get_mel_chunks(audio_path, fps=30, mel_step_size=16):
    y, sr = librosa.load(audio_path, sr=16000)
    mel = librosa.feature.melspectrogram(
        y=y,
        sr=sr,
        n_fft=800,
        hop_length=200,
        win_length=800,
        n_mels=80,
        fmin=55,
        fmax=7600
    )
    mel_db = librosa.power_to_db(mel, ref=np.max)
    mel_norm = np.clip((mel_db + 40) / 40.0, -1.0, 1.0)
    
    total_audio_sec = len(y) / sr
    total_frames = int(total_audio_sec * fps)
    
    mel_chunks = []
    mel_frames_per_sec = 16000 / 200.0
    
    for i in range(total_frames):
        center_mel = int(i * (mel_frames_per_sec / fps))
        start_mel = center_mel - mel_step_size // 2
        end_mel = start_mel + mel_step_size
        
        if start_mel < 0:
            pad_left = -start_mel
            chunk = mel_norm[:, 0:end_mel]
            chunk = np.pad(chunk, ((0, 0), (pad_left, 0)), mode='edge')
        elif end_mel > mel_norm.shape[1]:
            pad_right = end_mel - mel_norm.shape[1]
            chunk = mel_norm[:, start_mel:]
            chunk = np.pad(chunk, ((0, 0), (0, pad_right)), mode='edge')
        else:
            chunk = mel_norm[:, start_mel:end_mel]
            
        mel_chunks.append(chunk)
        
    return mel_chunks, total_frames, total_audio_sec

def generate_talking_host_video(
    image_path,
    audio_path,
    output_path,
    checkpoint_path="src/engine/wav2lip_repo/checkpoints/wav2lip_gan.pth",
    fps=30
):
    model, device = load_wav2lip_model(checkpoint_path)
    mel_chunks, total_frames, audio_dur = get_mel_chunks(audio_path, fps=fps)
    
    img = cv2.imread(image_path)
    h, w, _ = img.shape
    
    detector = cv2.FaceDetectorYN.create('artifacts/videos/face_detection_yunet.onnx', '', (w, h))
    _, faces = detector.detect(img)
    if faces is None or len(faces) == 0:
        raise ValueError("Could not detect face in host image.")
        
    face = faces[0]
    bbox = face[0:4].astype(int)
    fx, fy, fw, fh = bbox
    
    pad_y = int(fh * 0.25)
    pad_x = int(fw * 0.25)
    y1 = max(0, fy - pad_y)
    y2 = min(h, fy + fh + pad_y)
    x1 = max(0, fx - pad_x)
    x2 = min(w, fx + fw + pad_x)
    
    face_crop = img[y1:y2, x1:x2]
    orig_face_h, orig_face_w, _ = face_crop.shape
    
    face_96 = cv2.resize(face_crop, (96, 96))
    face_masked = face_96.copy()
    face_masked[48:, :] = 0
    
    face_input = np.concatenate([face_masked, face_96], axis=2) / 255.0
    face_tensor = torch.FloatTensor(face_input).permute(2, 0, 1).unsqueeze(0).to(device)
    
    right_eye = (int(face[4]), int(face[5]))
    left_eye = (int(face[6]), int(face[7]))
    eye_rad = int(fw * 0.15)
    
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    temp_raw_video = "artifacts/videos/living_video/temp_host_talking_raw.mp4"
    out = cv2.VideoWriter(temp_raw_video, fourcc, fps, (w, h))
    
    print(f"Generating {total_frames} frames of neural talking host video (Mouth-Only HD Blend)...")
    
    batch_size = 16
    generated_faces = []
    
    with torch.no_grad():
        for b in range(0, total_frames, batch_size):
            b_end = min(b + batch_size, total_frames)
            b_mels = mel_chunks[b:b_end]
            
            mel_batch = torch.FloatTensor(np.stack(b_mels, axis=0)).unsqueeze(1).to(device)
            face_batch = face_tensor.repeat(len(b_mels), 1, 1, 1)
            
            pred_faces = model(mel_batch, face_batch)
            pred_faces = pred_faces.permute(0, 2, 3, 1).cpu().numpy() * 255.0
            
            for p in pred_faces:
                generated_faces.append(np.clip(p, 0, 255).astype(np.uint8))
                
    # Mouth-only smooth elliptical mask (keeps eyes/glasses/nose 100% untouched)
    mask_96 = np.zeros((96, 96), dtype=np.float32)
    cv2.ellipse(mask_96, (48, 68), (26, 18), 0, 0, 360, 1.0, -1)
    mask_96 = cv2.GaussianBlur(mask_96, (11, 11), 0)[:, :, np.newaxis]
    mask_highres = cv2.resize(mask_96, (orig_face_w, orig_face_h), interpolation=cv2.INTER_LINEAR)[:, :, np.newaxis]
    
    f_cam = (w / 2.0) / math.tan(math.radians(55.0 / 2.0))
    cx, cy = w / 2.0, h / 2.0
    u_grid, v_grid = np.meshgrid(np.arange(w), np.arange(h))
    depth = np.ones((h, w), dtype=np.float32) * 0.6
    z_3d = 1.0 + (1.0 - depth) * 1.5
    
    for i in range(total_frames):
        t = i / float(total_frames)
        t_smooth = t * t * (3.0 - 2.0 * t)
        
        # 1. Blend only neural mouth region (HD preservation of face & glasses)
        pred_96 = generated_faces[i]
        pred_highres = cv2.resize(pred_96, (orig_face_w, orig_face_h), interpolation=cv2.INTER_LANCZOS4)
        
        blended_face = (pred_highres * mask_highres + face_crop * (1.0 - mask_highres)).astype(np.uint8)
        
        frame = img.copy()
        frame[y1:y2, x1:x2] = blended_face
        
        # 2. Eye blinking curve
        blink_val = 0.0
        for blink_center in [20, 65, 110]:
            diff = abs(i - blink_center)
            if diff < 4:
                blink_val = math.cos(diff / 4.0 * (math.pi / 2.0))
                break
                
        if blink_val > 0:
            for (ex, ey) in [left_eye, right_eye]:
                rad = int(eye_rad * 1.3)
                bx1, bx2 = max(0, ex - rad), min(w, ex + rad)
                by1, by2 = max(0, ey - rad), min(h, ey + rad)
                roi = frame[by1:by2, bx1:bx2].copy()
                rh, rw, _ = roi.shape
                if rh > 0 and rw > 0:
                    gy, gx = np.mgrid[0:rh, 0:rw]
                    norm_x = (gx - (ex - bx1)) / float(rad)
                    norm_y = (gy - (ey - by1)) / float(rad)
                    falloff = np.exp(-(norm_x**2 + norm_y**2) * 3.0)
                    dy = np.where(norm_y < 0.2, blink_val * 6.5 * falloff, 0)
                    map_x = gx.astype(np.float32)
                    map_y = np.clip(gy - dy, 0, rh - 1).astype(np.float32)
                    frame[by1:by2, bx1:bx2] = cv2.remap(roi, map_x, map_y, interpolation=cv2.INTER_CUBIC)
                    
        # 3. 3D Camera Orbit Rig Motion
        yaw = math.sin((t_smooth - 0.5) * math.pi * 0.8) * 2.8
        pitch = math.sin(t_smooth * math.pi) * 0.8
        roll = math.sin((t_smooth - 0.5) * math.pi) * 0.3
        tx = (t_smooth - 0.5) * 0.04
        ty = math.sin(t_smooth * math.pi) * 0.015
        tz = t_smooth * 0.05
        zoom = 1.20
        
        rad_yaw, rad_pitch, rad_roll = math.radians(yaw), math.radians(pitch), math.radians(roll)
        Ry = np.array([[math.cos(rad_yaw), 0, math.sin(rad_yaw)], [0, 1, 0], [-math.sin(rad_yaw), 0, math.cos(rad_yaw)]])
        Rx = np.array([[1, 0, 0], [0, math.cos(rad_pitch), -math.sin(rad_pitch)], [0, math.sin(rad_pitch), math.cos(rad_pitch)]])
        Rz = np.array([[math.cos(rad_roll), -math.sin(rad_roll), 0], [math.sin(rad_roll), math.cos(rad_roll), 0], [0, 0, 1]])
        R = Rz @ Rx @ Ry

        x_3d = ((u_grid - cx) / zoom) * z_3d / f_cam
        y_3d = ((v_grid - cy) / zoom) * z_3d / f_cam
        pts_3d = np.stack([x_3d, y_3d, z_3d], axis=-1)

        T = np.array([tx, ty, tz])
        pts_trans = pts_3d - T
        pts_rot = np.einsum('ij,hwj->hwi', R.T, pts_trans)

        z_rot = np.maximum(pts_rot[:, :, 2], 0.1)
        u_new = (pts_rot[:, :, 0] * f_cam / z_rot + cx).astype(np.float32)
        v_new = (pts_rot[:, :, 1] * f_cam / z_rot + cy).astype(np.float32)

        warped = cv2.remap(frame, u_new, v_new, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
        out.write(warped)
        
    out.release()
    print("Neural talking video generated successfully!")
    
    cmd = f"/opt/homebrew/bin/ffmpeg -y -i {temp_raw_video} -i {audio_path} -c:v libx264 -crf 17 -pix_fmt yuv420p -c:a aac -b:a 192k -shortest {output_path}"
    os.system(cmd)
    print(f"Master neural talking video saved to {output_path}")

if __name__ == "__main__":
    generate_talking_host_video(
        image_path="artifacts/videos/shot_01_host.jpg",
        audio_path="artifacts/videos/affiliate_demo_voiceover.mp3",
        output_path="artifacts/videos/living_video/master_neural_talking_host.mp4",
        fps=30
    )
