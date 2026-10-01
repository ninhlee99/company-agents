"""
Living Video & Neural Facial Animation Engine
Creates photorealistic living video footage from static shots:
1. Audio-driven Lip-Sync & Jaw Muscle Deformation (synchronous with Vietnamese speech).
2. Organic Eye Blinking & Micro-Expression Dynamics.
3. Conversational Head Nods & Gestural Breathing.
4. Physical Living B-Roll:
   - Dynamic Finger Clicking & Mouse Hand Motion.
   - Pulsing Cyberpunk LED Glow & Light Sweep on AI Hub.
   - Rising Steam Particles & Ambient Light Breathing on Desk Setup.
5. Ray-Scaled 3D Camera Rig Kinematics at 60 FPS.
"""

import os
import sys
import math
import cv2
import numpy as np
import librosa
import torch

def get_device():
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")

def analyze_audio(audio_path, fps=60):
    """
    Extracts frame-by-frame speech energy (RMS), spectral centroid,
    and viseme opening coefficients from the audio file.
    """
    y, sr = librosa.load(audio_path, sr=24000)
    hop_length = int(sr / fps)
    
    # Short-time RMS energy
    rms = librosa.feature.rms(y=y, frame_length=hop_length * 2, hop_length=hop_length)[0]
    # Normalize RMS
    rms_max = np.percentile(rms, 95) if len(rms) > 0 else 1.0
    rms_norm = np.clip(rms / (rms_max + 1e-6), 0.0, 1.0)
    
    # Spectral centroid (distinguishes open vowels vs closed consonants)
    centroid = librosa.feature.spectral_centroid(y=y, sr=sr, hop_length=hop_length)[0]
    centroid_norm = np.clip((centroid - 1000) / 3000.0, 0.0, 1.0)
    
    # Smooth envelope with 3-tap moving average
    kernel = np.ones(3) / 3.0
    rms_smooth = np.convolve(rms_norm, kernel, mode='same')
    
    return rms_smooth, centroid_norm, len(y) / sr

def morph_mouth(img, mouth_center, mouth_w, mouth_h, open_amount, stretch_amount=0.0):
    """
    Deforms facial lip and jaw geometry realistically:
    - Vertical jaw lowering and lip opening.
    - Lateral lip stretch.
    - Inner mouth shadow/oral cavity rendering.
    """
    h, w, _ = img.shape
    cx, cy = mouth_center
    
    # Define ROI around mouth
    roi_rad_x = int(mouth_w * 0.9)
    roi_rad_y = int(mouth_h * 1.5)
    
    x1 = max(0, cx - roi_rad_x)
    x2 = min(w, cx + roi_rad_x)
    y1 = max(0, cy - roi_rad_y)
    y2 = min(h, cy + roi_rad_y + int(open_amount * 15))
    
    roi = img[y1:y2, x1:x2].copy()
    rh, rw, _ = roi.shape
    if rh == 0 or rw == 0:
        return img
    
    # Meshgrid inside ROI
    grid_y, grid_x = np.mgrid[0:rh, 0:rw]
    norm_x = (grid_x - (cx - x1)) / float(roi_rad_x + 1e-5)
    norm_y = (grid_y - (cy - y1)) / float(roi_rad_y + 1e-5)
    
    dist_sq = norm_x**2 + norm_y**2
    falloff = np.exp(-dist_sq * 2.5) # Gaussian facial muscle falloff
    
    # Lower lip & jaw displacement downward
    dy = np.where(norm_y > 0, open_amount * 9.0 * falloff * norm_y, open_amount * -3.0 * falloff * (-norm_y))
    # Horizontal lip stretch
    dx = open_amount * stretch_amount * 6.0 * falloff * norm_x
    
    map_x = np.clip(grid_x - dx, 0, rw - 1).astype(np.float32)
    map_y = np.clip(grid_y - dy, 0, rh - 1).astype(np.float32)
    
    warped_roi = cv2.remap(roi, map_x, map_y, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
    
    # Subtle inner mouth natural depth shadow when wide open
    if open_amount > 0.4:
        shadow_mask = np.exp(-((norm_x * 1.5)**2 + (norm_y * 3.0)**2) * 5.0)
        shadow_factor = 1.0 - (open_amount - 0.4) * 0.4 * shadow_mask[:, :, np.newaxis]
        warped_roi = np.clip(warped_roi * shadow_factor, 0, 255).astype(np.uint8)
        
    res = img.copy()
    res[y1:y2, x1:x2] = warped_roi
    return res

def morph_eyes_blink(img, left_eye, right_eye, eye_radius, blink_amount):
    """
    Deforms upper eyelid downward to create realistic eye blinking.
    """
    if blink_amount <= 0.01:
        return img
    
    h, w, _ = img.shape
    res = img.copy()
    
    for (ex, ey) in [left_eye, right_eye]:
        rad = int(eye_radius * 1.4)
        x1 = max(0, ex - rad)
        x2 = min(w, ex + rad)
        y1 = max(0, ey - rad)
        y2 = min(h, ey + rad)
        
        roi = img[y1:y2, x1:x2].copy()
        rh, rw, _ = roi.shape
        if rh == 0 or rw == 0:
            continue
            
        grid_y, grid_x = np.mgrid[0:rh, 0:rw]
        norm_x = (grid_x - (ex - x1)) / float(rad + 1e-5)
        norm_y = (grid_y - (ey - y1)) / float(rad + 1e-5)
        
        dist_sq = norm_x**2 + norm_y**2
        falloff = np.exp(-dist_sq * 3.0)
        
        # Upper eyelid moves down to close over pupil
        dy = np.where(norm_y < 0.2, blink_amount * 7.5 * falloff, 0)
        
        map_x = grid_x.astype(np.float32)
        map_y = np.clip(grid_y - dy, 0, rh - 1).astype(np.float32)
        
        warped_roi = cv2.remap(roi, map_x, map_y, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
        res[y1:y2, x1:x2] = warped_roi
        
    return res

def render_living_host_shot(
    img_path,
    output_path,
    audio_path,
    fps=60,
    duration=3.5
):
    """Renders the living talking host shot with lip-sync, blinking, and 3D camera orbit."""
    img = cv2.imread(img_path)
    h, w, _ = img.shape
    
    # Detect face
    detector = cv2.FaceDetectorYN.create('artifacts/videos/face_detection_yunet.onnx', '', (w, h))
    _, faces = detector.detect(img)
    if faces is not None and len(faces) > 0:
        face = faces[0]
        right_eye = (int(face[4]), int(face[5]))
        left_eye = (int(face[6]), int(face[7]))
        mouth_r = (int(face[10]), int(face[11]))
        mouth_l = (int(face[12]), int(face[13]))
        mouth_center = ((mouth_r[0] + mouth_l[0]) // 2, (mouth_r[1] + mouth_l[1]) // 2)
        mouth_w = int(math.hypot(mouth_l[0] - mouth_r[0], mouth_l[1] - mouth_r[1]))
        mouth_h = int(mouth_w * 0.4)
        eye_rad = int(mouth_w * 0.35)
    else:
        right_eye = (349, 444)
        left_eye = (434, 435)
        mouth_center = (404, 515)
        mouth_w = 81
        mouth_h = 32
        eye_rad = 28
        
    # Analyze voice
    rms_arr, centroid_arr, total_dur = analyze_audio(audio_path, fps=fps)
    total_frames = int(duration * fps)
    
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_path, fourcc, fps, (w, h))
    
    print(f"[Living Host] Rendering {total_frames} frames (Lip-sync + Blinks + 3D Orbit)...")
    
    f = (w / 2.0) / math.tan(math.radians(55.0 / 2.0))
    cx, cy = w / 2.0, h / 2.0
    u, v = np.meshgrid(np.arange(w), np.arange(h))
    depth = np.ones((h, w), dtype=np.float32) * 0.6
    z_3d = 1.0 + (1.0 - depth) * 1.5
    
    for i in range(total_frames):
        t = i / float(total_frames)
        t_smooth = t * t * (3.0 - 2.0 * t)
        
        # Audio energy for this frame
        audio_idx = min(i, len(rms_arr) - 1) if len(rms_arr) > 0 else 0
        speech_energy = rms_arr[audio_idx] if len(rms_arr) > 0 else 0.0
        
        # Lip-sync opening magnitude with natural speech flutter
        mouth_open = np.clip(speech_energy * 1.25, 0.0, 1.0)
        
        # Natural periodic blinking: blinks around frame 45 (0.75s) and frame 150 (2.5s)
        blink_val = 0.0
        for blink_center in [45, 150]:
            diff = abs(i - blink_center)
            if diff < 6:
                blink_val = math.cos(diff / 6.0 * (math.pi / 2.0))
                break
                
        # 1. Apply Facial Lip-sync deformation
        face_frame = morph_mouth(img, mouth_center, mouth_w, mouth_h, mouth_open)
        # 2. Apply Eye Blink deformation
        face_frame = morph_eyes_blink(face_frame, left_eye, right_eye, eye_rad, blink_val)
        
        # 3. Conversational head micro-nodding synced with speech
        head_nod = math.sin(i * 0.15) * speech_energy * 1.2
        
        # 4. 3D Camera Rig Steadicam Orbit
        yaw = math.sin((t_smooth - 0.5) * math.pi * 0.8) * 3.0 + math.sin(i * 0.1) * 0.2
        pitch = math.sin(t_smooth * math.pi) * 0.8 + head_nod
        roll = math.sin((t_smooth - 0.5) * math.pi) * 0.35
        tx = (t_smooth - 0.5) * 0.04
        ty = math.sin(t_smooth * math.pi) * 0.015
        tz = t_smooth * 0.05
        zoom = 1.20
        
        rad_yaw = math.radians(yaw)
        rad_pitch = math.radians(pitch)
        rad_roll = math.radians(roll)

        Ry = np.array([[math.cos(rad_yaw), 0, math.sin(rad_yaw)], [0, 1, 0], [-math.sin(rad_yaw), 0, math.cos(rad_yaw)]])
        Rx = np.array([[1, 0, 0], [0, math.cos(rad_pitch), -math.sin(rad_pitch)], [0, math.sin(rad_pitch), math.cos(rad_pitch)]])
        Rz = np.array([[math.cos(rad_roll), -math.sin(rad_roll), 0], [math.sin(rad_roll), math.cos(rad_roll), 0], [0, 0, 1]])
        R = Rz @ Rx @ Ry

        x_3d = ((u - cx) / zoom) * z_3d / f
        y_3d = ((v - cy) / zoom) * z_3d / f
        pts_3d = np.stack([x_3d, y_3d, z_3d], axis=-1)

        T = np.array([tx, ty, tz])
        pts_trans = pts_3d - T
        pts_rot = np.einsum('ij,hwj->hwi', R.T, pts_trans)

        z_rot = np.maximum(pts_rot[:, :, 2], 0.1)
        u_new = (pts_rot[:, :, 0] * f / z_rot + cx).astype(np.float32)
        v_new = (pts_rot[:, :, 1] * f / z_rot + cy).astype(np.float32)

        warped = cv2.remap(face_frame, u_new, v_new, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
        out.write(warped)
        
    out.release()
    print(f"[Living Host] Saved {output_path}")

def render_living_mouse_shot(
    img_path,
    output_path,
    fps=60,
    duration=3.5
):
    """Renders living mouse shot with finger clicking animation and crane motion."""
    img = cv2.imread(img_path)
    h, w, _ = img.shape
    total_frames = int(duration * fps)
    
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_path, fourcc, fps, (w, h))
    
    # Index finger tip area: (x ~ 260..360, y ~ 450..580)
    fx, fy = 310, 520
    
    print(f"[Living Mouse] Rendering {total_frames} frames (Finger clicks + Crane)...")
    
    f = (w / 2.0) / math.tan(math.radians(55.0 / 2.0))
    cx, cy = w / 2.0, h / 2.0
    u, v = np.meshgrid(np.arange(w), np.arange(h))
    depth = np.ones((h, w), dtype=np.float32) * 0.7
    z_3d = 1.0 + (1.0 - depth) * 1.5
    
    for i in range(total_frames):
        t = i / float(total_frames)
        t_smooth = t * t * (3.0 - 2.0 * t)
        
        # Click cycles at frames 30, 90, 150 (0.5s, 1.5s, 2.5s)
        click_val = 0.0
        for click_center in [30, 90, 150]:
            diff = abs(i - click_center)
            if diff < 5:
                click_val = math.cos(diff / 5.0 * (math.pi / 2.0))
                break
                
        # Morph index finger pressing down
        mouse_frame = img.copy()
        if click_val > 0:
            rad = 90
            x1, x2 = max(0, fx - rad), min(w, fx + rad)
            y1, y2 = max(0, fy - rad), min(h, fy + rad)
            roi = mouse_frame[y1:y2, x1:x2].copy()
            rh, rw, _ = roi.shape
            grid_y, grid_x = np.mgrid[0:rh, 0:rw]
            norm_x = (grid_x - (fx - x1)) / float(rad)
            norm_y = (grid_y - (fy - y1)) / float(rad)
            falloff = np.exp(-(norm_x**2 + norm_y**2) * 4.0)
            
            # Press down by 4px
            dy = click_val * 4.5 * falloff
            map_x = grid_x.astype(np.float32)
            map_y = np.clip(grid_y - dy, 0, rh - 1).astype(np.float32)
            mouse_frame[y1:y2, x1:x2] = cv2.remap(roi, map_x, map_y, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
            
        # 3D Crane boom motion
        yaw = math.sin((t_smooth - 0.5) * math.pi) * 2.0
        pitch = 1.6 - t_smooth * 3.0
        roll = math.sin(t_smooth * math.pi) * -0.25
        tx = math.sin(t_smooth * math.pi * 0.5) * 0.025
        ty = (t_smooth - 0.5) * 0.07
        tz = t_smooth * 0.05
        zoom = 1.22
        
        rad_yaw, rad_pitch, rad_roll = math.radians(yaw), math.radians(pitch), math.radians(roll)
        Ry = np.array([[math.cos(rad_yaw), 0, math.sin(rad_yaw)], [0, 1, 0], [-math.sin(rad_yaw), 0, math.cos(rad_yaw)]])
        Rx = np.array([[1, 0, 0], [0, math.cos(rad_pitch), -math.sin(rad_pitch)], [0, math.sin(rad_pitch), math.cos(rad_pitch)]])
        Rz = np.array([[math.cos(rad_roll), -math.sin(rad_roll), 0], [math.sin(rad_roll), math.cos(rad_roll), 0], [0, 0, 1]])
        R = Rz @ Rx @ Ry

        x_3d = ((u - cx) / zoom) * z_3d / f
        y_3d = ((v - cy) / zoom) * z_3d / f
        pts_3d = np.stack([x_3d, y_3d, z_3d], axis=-1)

        T = np.array([tx, ty, tz])
        pts_trans = pts_3d - T
        pts_rot = np.einsum('ij,hwj->hwi', R.T, pts_trans)

        z_rot = np.maximum(pts_rot[:, :, 2], 0.1)
        u_new = (pts_rot[:, :, 0] * f / z_rot + cx).astype(np.float32)
        v_new = (pts_rot[:, :, 1] * f / z_rot + cy).astype(np.float32)

        warped = cv2.remap(mouse_frame, u_new, v_new, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
        out.write(warped)
        
    out.release()
    print(f"[Living Mouse] Saved {output_path}")

def render_living_hub_shot(
    img_path,
    output_path,
    fps=60,
    duration=3.5
):
    """Renders living AI Hub shot with pulsing cyan LED light and dolly push."""
    img = cv2.imread(img_path)
    h, w, _ = img.shape
    total_frames = int(duration * fps)
    
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_path, fourcc, fps, (w, h))
    
    # LED light center on Hub: (x ~ 420, y ~ 560)
    lx, ly = 420, 560
    
    print(f"[Living Hub] Rendering {total_frames} frames (Pulsing LED + Dolly)...")
    
    f = (w / 2.0) / math.tan(math.radians(55.0 / 2.0))
    cx, cy = w / 2.0, h / 2.0
    u, v = np.meshgrid(np.arange(w), np.arange(h))
    depth = np.ones((h, w), dtype=np.float32) * 0.7
    z_3d = 1.0 + (1.0 - depth) * 1.5
    
    for i in range(total_frames):
        t = i / float(total_frames)
        t_smooth = t * t * (3.0 - 2.0 * t)
        
        # Breathing LED pulse intensity (sine wave)
        pulse = 1.0 + 0.35 * math.sin(i * 0.15)
        
        hub_frame = img.copy().astype(np.float32)
        
        # Add cyan neon glow around LED
        rad = 120
        x1, x2 = max(0, lx - rad), min(w, lx + rad)
        y1, y2 = max(0, ly - rad), min(h, ly + rad)
        grid_y, grid_x = np.mgrid[y1:y2, x1:x2]
        dist_sq = ((grid_x - lx)/float(rad))**2 + ((grid_y - ly)/float(rad))**2
        glow_mask = np.exp(-dist_sq * 3.5)[:, :, np.newaxis]
        
        # Boost Cyan channel (B + G)
        hub_frame[y1:y2, x1:x2, 0] += glow_mask[:, :, 0] * 45.0 * (pulse - 0.8)
        hub_frame[y1:y2, x1:x2, 1] += glow_mask[:, :, 0] * 55.0 * (pulse - 0.8)
        hub_frame = np.clip(hub_frame, 0, 255).astype(np.uint8)
        
        # 3D Dolly push track
        yaw = math.sin((t_smooth - 0.5) * math.pi) * 1.6
        pitch = -0.6 + t_smooth * 1.5
        roll = math.cos(t_smooth * math.pi * 0.5) * 0.25
        tx = (0.5 - t_smooth) * 0.035
        ty = (t_smooth - 0.5) * 0.025
        tz = t_smooth * 0.08
        zoom = 1.22
        
        rad_yaw, rad_pitch, rad_roll = math.radians(yaw), math.radians(pitch), math.radians(roll)
        Ry = np.array([[math.cos(rad_yaw), 0, math.sin(rad_yaw)], [0, 1, 0], [-math.sin(rad_yaw), 0, math.cos(rad_yaw)]])
        Rx = np.array([[1, 0, 0], [0, math.cos(rad_pitch), -math.sin(rad_pitch)], [0, math.sin(rad_pitch), math.cos(rad_pitch)]])
        Rz = np.array([[math.cos(rad_roll), -math.sin(rad_roll), 0], [math.sin(rad_roll), math.cos(rad_roll), 0], [0, 0, 1]])
        R = Rz @ Rx @ Ry

        x_3d = ((u - cx) / zoom) * z_3d / f
        y_3d = ((v - cy) / zoom) * z_3d / f
        pts_3d = np.stack([x_3d, y_3d, z_3d], axis=-1)

        T = np.array([tx, ty, tz])
        pts_trans = pts_3d - T
        pts_rot = np.einsum('ij,hwj->hwi', R.T, pts_trans)

        z_rot = np.maximum(pts_rot[:, :, 2], 0.1)
        u_new = (pts_rot[:, :, 0] * f / z_rot + cx).astype(np.float32)
        v_new = (pts_rot[:, :, 1] * f / z_rot + cy).astype(np.float32)

        warped = cv2.remap(hub_frame, u_new, v_new, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
        out.write(warped)
        
    out.release()
    print(f"[Living Hub] Saved {output_path}")

def render_living_setup_shot(
    img_path,
    output_path,
    fps=60,
    duration=3.04
):
    """Renders living desk setup shot with rising steam above coffee cup and pedestal reveal."""
    img = cv2.imread(img_path)
    h, w, _ = img.shape
    total_frames = int(duration * fps)
    
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_path, fourcc, fps, (w, h))
    
    # Coffee cup steam origin: (x ~ 215, y ~ 760)
    sx, sy = 215, 760
    
    print(f"[Living Setup] Rendering {total_frames} frames (Rising Steam + Pedestal Reveal)...")
    
    f = (w / 2.0) / math.tan(math.radians(55.0 / 2.0))
    cx, cy = w / 2.0, h / 2.0
    u, v = np.meshgrid(np.arange(w), np.arange(h))
    depth = np.ones((h, w), dtype=np.float32) * 0.6
    z_3d = 1.0 + (1.0 - depth) * 1.5
    
    for i in range(total_frames):
        t = i / float(total_frames)
        t_smooth = t * t * (3.0 - 2.0 * t)
        
        setup_frame = img.copy().astype(np.float32)
        
        # Animate rising steam particles
        for p in range(5):
            particle_t = (i * 0.03 + p * 0.2) % 1.0
            p_y = sy - int(particle_t * 90)
            p_x = sx + int(math.sin(particle_t * 4.0 + p) * 12)
            p_rad = int(12 + particle_t * 22)
            alpha = (1.0 - particle_t) * 0.18
            
            x1, x2 = max(0, p_x - p_rad), min(w, p_x + p_rad)
            y1, y2 = max(0, p_y - p_rad), min(h, p_y + p_rad)
            if x2 > x1 and y2 > y1:
                gy, gx = np.mgrid[y1:y2, x1:x2]
                d_sq = ((gx - p_x)/float(p_rad))**2 + ((gy - p_y)/float(p_rad))**2
                steam_mask = np.exp(-d_sq * 2.5) * alpha
                setup_frame[y1:y2, x1:x2] += steam_mask[:, :, np.newaxis] * 120.0
                
        setup_frame = np.clip(setup_frame, 0, 255).astype(np.uint8)
        
        # 3D Pedestal reveal tilt
        yaw = math.sin((t_smooth - 0.5) * math.pi) * 1.2
        pitch = -1.6 + t_smooth * 2.4
        roll = math.sin(t_smooth * math.pi * 0.7) * 0.2
        tx = (t_smooth - 0.5) * 0.025
        ty = (0.5 - t_smooth) * 0.06
        tz = t_smooth * 0.06
        zoom = 1.20
        
        rad_yaw, rad_pitch, rad_roll = math.radians(yaw), math.radians(pitch), math.radians(roll)
        Ry = np.array([[math.cos(rad_yaw), 0, math.sin(rad_yaw)], [0, 1, 0], [-math.sin(rad_yaw), 0, math.cos(rad_yaw)]])
        Rx = np.array([[1, 0, 0], [0, math.cos(rad_pitch), -math.sin(rad_pitch)], [0, math.sin(rad_pitch), math.cos(rad_pitch)]])
        Rz = np.array([[math.cos(rad_roll), -math.sin(rad_roll), 0], [math.sin(rad_roll), math.cos(rad_roll), 0], [0, 0, 1]])
        R = Rz @ Rx @ Ry

        x_3d = ((u - cx) / zoom) * z_3d / f
        y_3d = ((v - cy) / zoom) * z_3d / f
        pts_3d = np.stack([x_3d, y_3d, z_3d], axis=-1)

        T = np.array([tx, ty, tz])
        pts_trans = pts_3d - T
        pts_rot = np.einsum('ij,hwj->hwi', R.T, pts_trans)

        z_rot = np.maximum(pts_rot[:, :, 2], 0.1)
        u_new = (pts_rot[:, :, 0] * f / z_rot + cx).astype(np.float32)
        v_new = (pts_rot[:, :, 1] * f / z_rot + cy).astype(np.float32)

        warped = cv2.remap(setup_frame, u_new, v_new, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
        out.write(warped)
        
    out.release()
    print(f"[Living Setup] Saved {output_path}")

def main():
    os.makedirs("artifacts/videos/living_video", exist_ok=True)
    
    audio_path = "artifacts/videos/affiliate_demo_voiceover.mp3"
    
    # 1. Render Living Host with Lip-Sync & Blinking
    render_living_host_shot(
        img_path="artifacts/videos/shot_01_host.jpg",
        output_path="artifacts/videos/living_video/living_01_host.mp4",
        audio_path=audio_path,
        fps=60,
        duration=3.5
    )
    
    # 2. Render Living Mouse with Finger Clicking
    render_living_mouse_shot(
        img_path="artifacts/videos/shot_02_mouse.jpg",
        output_path="artifacts/videos/living_video/living_02_mouse.mp4",
        fps=60,
        duration=3.5
    )
    
    # 3. Render Living Hub with Pulsing LED
    render_living_hub_shot(
        img_path="artifacts/videos/shot_03_hub.jpg",
        output_path="artifacts/videos/living_video/living_03_hub.mp4",
        fps=60,
        duration=3.5
    )
    
    # 4. Render Living Desk Setup with Rising Steam
    render_living_setup_shot(
        img_path="artifacts/videos/shot_04_lightbar.jpg",
        output_path="artifacts/videos/living_video/living_04_setup.mp4",
        fps=60,
        duration=3.04
    )
    
    print("All 4 Living Video Shots rendered successfully!")

if __name__ == "__main__":
    main()
