"""
Cinematic 3D Video & Camera Motion Engine (v3 - Studio Master)
- High-precision Ray-Scaled 3D Projection (Zero Boundary Artifacts)
- Neural Depth-driven 3D Parallax (MiDaS / Apple Silicon Metal MPS)
- Cinematography Rigs:
  * Shot 1: Steadicam Orbit Arc (Sweeping 3D curve around Host)
  * Shot 2: Crane Boom Down (Overhead boom focusing on hands & mouse)
  * Shot 3: Dolly-Track Push (High-precision slider push on Hub)
  * Shot 4: Pedestal Reveal Tilt (Upward pedestal reveal of cozy desk setup)
- Sub-shutter Motion Blur & 60 FPS Ultra-Smooth Rendering
"""

import os
import sys
import math
import cv2
import numpy as np
import torch

def get_device():
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")

def load_depth_model():
    device = get_device()
    try:
        model_type = "MiDaS_small"
        midas = torch.hub.load("intel-isl/MiDaS", model_type, trust_repo=True)
        midas.to(device)
        midas.eval()
        midas_transforms = torch.hub.load("intel-isl/MiDaS", "transforms", trust_repo=True)
        transform = midas_transforms.small_transform
        print(f"MiDaS Neural Depth Model loaded successfully on {device}")
        return midas, transform, device
    except Exception as e:
        print(f"Warning: MiDaS load fallback ({e})")
        return None, None, device

def estimate_depth(img_rgb, model, transform, device):
    h, w, _ = img_rgb.shape
    if model is not None and transform is not None:
        try:
            input_batch = transform(img_rgb).to(device)
            with torch.no_grad():
                prediction = model(input_batch)
                prediction = torch.nn.functional.interpolate(
                    prediction.unsqueeze(1),
                    size=(h, w),
                    mode="bicubic",
                    align_corners=False,
                ).squeeze()
            depth = prediction.cpu().numpy()
            d_min, d_max = np.percentile(depth, 3), np.percentile(depth, 97)
            depth = np.clip((depth - d_min) / (d_max - d_min + 1e-6), 0.0, 1.0)
            depth = cv2.bilateralFilter(depth.astype(np.float32), d=7, sigmaColor=0.1, sigmaSpace=7)
            return depth
        except Exception as e:
            print(f"Depth inference fallback: {e}")

    # Fallback saliency gradient depth
    y_coords, x_coords = np.mgrid[0:h, 0:w]
    norm_y = y_coords / float(h)
    norm_x = (x_coords - w / 2.0) / (w / 2.0)
    center_dist = np.sqrt(norm_x**2 + (norm_y - 0.55)**2)
    radial_depth = 1.0 - np.clip(center_dist * 0.7, 0.0, 1.0)
    return cv2.GaussianBlur(radial_depth.astype(np.float32), (31, 31), 0)

def render_3d_frame_clean(img, depth, yaw, pitch, roll, tx, ty, tz, zoom=1.22, fov=55.0):
    """
    Renders a 3D frame using ray scaling to ensure zero boundary corruption.
    """
    h, w, _ = img.shape
    f = (w / 2.0) / math.tan(math.radians(fov / 2.0))
    cx = w / 2.0
    cy = h / 2.0

    rad_yaw = math.radians(yaw)
    rad_pitch = math.radians(pitch)
    rad_roll = math.radians(roll)

    Ry = np.array([
        [math.cos(rad_yaw), 0, math.sin(rad_yaw)],
        [0, 1, 0],
        [-math.sin(rad_yaw), 0, math.cos(rad_yaw)]
    ])
    Rx = np.array([
        [1, 0, 0],
        [0, math.cos(rad_pitch), -math.sin(rad_pitch)],
        [0, math.sin(rad_pitch), math.cos(rad_pitch)]
    ])
    Rz = np.array([
        [math.cos(rad_roll), -math.sin(rad_roll), 0],
        [math.sin(rad_roll), math.cos(rad_roll), 0],
        [0, 0, 1]
    ])
    R = Rz @ Rx @ Ry

    u, v = np.meshgrid(np.arange(w), np.arange(h))
    
    # Scale depth (0.1 to 1.0)
    z_3d = 1.0 + (1.0 - depth) * 1.5
    
    # Ray scale to fit in sensor canvas
    x_3d = ((u - cx) / zoom) * z_3d / f
    y_3d = ((v - cy) / zoom) * z_3d / f
    pts_3d = np.stack([x_3d, y_3d, z_3d], axis=-1)
    
    T = np.array([tx, ty, tz])
    pts_trans = pts_3d - T
    pts_rot = np.einsum('ij,hwj->hwi', R.T, pts_trans)
    
    z_rot = np.maximum(pts_rot[:, :, 2], 0.1)
    u_new = (pts_rot[:, :, 0] * f / z_rot + cx).astype(np.float32)
    v_new = (pts_rot[:, :, 1] * f / z_rot + cy).astype(np.float32)
    
    warped = cv2.remap(img, u_new, v_new, interpolation=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
    return warped

def generate_cinematic_clip(
    image_path,
    output_path,
    shot_type="orbit_arc",
    duration_sec=3.5,
    fps=60,
    model=None,
    transform=None,
    device="cpu"
):
    img_bgr = cv2.imread(image_path)
    if img_bgr is None:
        raise ValueError(f"Could not load image at {image_path}")
    
    img_rgb = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2RGB)
    print(f"[{shot_type}] Estimating neural 3D depth for {os.path.basename(image_path)}...")
    depth = estimate_depth(img_rgb, model, transform, device)
    
    total_frames = int(duration_sec * fps)
    h, w, _ = img_bgr.shape
    
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_path, fourcc, fps, (w, h))
    
    print(f"[{shot_type}] Rendering {total_frames} frames (60 FPS) with 3D camera rig...")
    
    for i in range(total_frames):
        sub_samples = 3
        accum_frame = np.zeros_like(img_bgr, dtype=np.float32)
        
        for s in range(sub_samples):
            sub_t = (i + s / float(sub_samples)) / float(total_frames)
            sub_smooth = sub_t * sub_t * (3.0 - 2.0 * sub_t)
            
            if shot_type == "orbit_arc":
                # Steadicam Orbit: smooth sweeping curve around host
                yaw = math.sin((sub_smooth - 0.5) * math.pi * 0.8) * 3.2
                pitch = math.sin(sub_smooth * math.pi) * 1.0
                roll = math.sin((sub_smooth - 0.5) * math.pi) * 0.4
                tx = (sub_smooth - 0.5) * 0.05
                ty = math.sin(sub_smooth * math.pi) * 0.015
                tz = sub_smooth * 0.06
                zoom = 1.20
            
            elif shot_type == "crane_boom":
                # Crane boom down onto product with subtle pitch shift
                yaw = math.sin((sub_smooth - 0.5) * math.pi) * 2.2
                pitch = 1.8 - sub_smooth * 3.2
                roll = math.sin(sub_smooth * math.pi) * -0.3
                tx = math.sin(sub_smooth * math.pi * 0.5) * 0.03
                ty = (sub_smooth - 0.5) * 0.08
                tz = sub_smooth * 0.06
                zoom = 1.22
                
            elif shot_type == "dolly_track":
                # Dolly push track shot
                yaw = math.sin((sub_smooth - 0.5) * math.pi) * 1.8
                pitch = -0.8 + sub_smooth * 1.8
                roll = math.cos(sub_smooth * math.pi * 0.5) * 0.3
                tx = (0.5 - sub_smooth) * 0.04
                ty = (sub_smooth - 0.5) * 0.03
                tz = sub_smooth * 0.10
                zoom = 1.22
                
            elif shot_type == "reveal_tilt":
                # Upward pedestal reveal
                yaw = math.sin((sub_smooth - 0.5) * math.pi) * 1.5
                pitch = -2.0 + sub_smooth * 2.8
                roll = math.sin(sub_smooth * math.pi * 0.7) * 0.25
                tx = (sub_smooth - 0.5) * 0.03
                ty = (0.5 - sub_smooth) * 0.07
                tz = sub_smooth * 0.08
                zoom = 1.20
            
            else:
                yaw, pitch, roll, tx, ty, tz = 0, 0, 0, 0, 0, 0
                zoom = 1.15
            
            micro_jitter_x = math.sin(i * 0.3 + s * 0.1) * 0.0005
            micro_jitter_y = math.cos(i * 0.25 + s * 0.1) * 0.0005
            
            rendered_sub = render_3d_frame_clean(
                img_bgr,
                depth,
                yaw=yaw,
                pitch=pitch,
                roll=roll,
                tx=tx + micro_jitter_x,
                ty=ty + micro_jitter_y,
                tz=tz,
                zoom=zoom
            )
            accum_frame += rendered_sub.astype(np.float32)
        
        frame_final = np.clip(accum_frame / sub_samples, 0, 255).astype(np.uint8)
        out.write(frame_final)
    
    out.release()
    print(f"[{shot_type}] Successfully rendered {output_path}")

def main():
    os.makedirs("artifacts/videos/cinematic_3d", exist_ok=True)
    
    model, transform, device = load_depth_model()
    
    shots = [
        {
            "img": "artifacts/videos/shot_01_host.jpg",
            "out": "artifacts/videos/cinematic_3d/clip_01_host_3d.mp4",
            "type": "orbit_arc",
            "duration": 3.5
        },
        {
            "img": "artifacts/videos/shot_02_mouse.jpg",
            "out": "artifacts/videos/cinematic_3d/clip_02_mouse_3d.mp4",
            "type": "crane_boom",
            "duration": 3.5
        },
        {
            "img": "artifacts/videos/shot_03_hub.jpg",
            "out": "artifacts/videos/cinematic_3d/clip_03_hub_3d.mp4",
            "type": "dolly_track",
            "duration": 3.5
        },
        {
            "img": "artifacts/videos/shot_04_lightbar.jpg",
            "out": "artifacts/videos/cinematic_3d/clip_04_setup_3d.mp4",
            "type": "reveal_tilt",
            "duration": 3.04
        }
    ]
    
    for shot in shots:
        generate_cinematic_clip(
            image_path=shot["img"],
            output_path=shot["out"],
            shot_type=shot["type"],
            duration_sec=shot["duration"],
            fps=60,
            model=model,
            transform=transform,
            device=device
        )
    
    print("Master 3D clips rendered cleanly.")

if __name__ == "__main__":
    main()
