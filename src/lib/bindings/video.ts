import { invoke } from "@tauri-apps/api/core";

export const setBrightness = (value: number) => invoke<void>("set_brightness", { value });
export const setContrast = (value: number) => invoke<void>("set_contrast", { value });
export const setSaturation = (value: number) => invoke<void>("set_saturation", { value });
export const toggleDeinterlace = () => invoke<void>("toggle_deinterlace");
export const setVideoZoom = (value: number) => invoke<void>("set_video_zoom", { value });
export const setVideoPan = (x: number, y: number) => invoke<void>("set_video_pan", { x, y });
export const resetVideoZoomPan = () => invoke<void>("reset_video_zoom_pan");

export interface ZoomPanState { zoom: number; pan_x: number; pan_y: number; }
export const getVideoZoomPan = () => invoke<ZoomPanState>("get_video_zoom_pan");

export const setAspectRatio = (ratio: string) => invoke<void>("set_aspect_ratio", { ratio });
export const getAspectRatio = () => invoke<string>("get_aspect_ratio");
