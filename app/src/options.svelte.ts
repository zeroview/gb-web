import { type Palette, type EmulatorOptions } from "DMG-2025";

export const palettes: Record<string, Palette> = {
  LCD: [
    [0.327778, 0.5028864, 0.0047769, 1.0],
    [0.2581828, 0.4125426, 0.0047769, 1.0],
    [0.0295568, 0.1221387, 0.0295568, 1.0],
    [0.0047769, 0.0395462, 0.0047769, 1.0],
  ],
  Clear: [
    [0.7454042, 0.9386857, 0.6307571, 1.0],
    [0.2462013, 0.5271151, 0.1620293, 1.0],
    [0.0343398, 0.1384316, 0.0930589, 1.0],
    [0.0024282, 0.009134, 0.0144438, 1.0],
  ],
  Raw: [
    [1.0, 1.0, 1.0, 1.0],
    [0.6666, 0.6666, 0.6666, 1.0],
    [0.3333, 0.3333, 0.3333, 1.0],
    [0.0, 0.0, 0.0, 1.0],
  ],
};
export const paletteNames = ["LCD", "Clear", "Raw"]

export const defaultOptions = {
  paletteIndex: 0,
  speed: 1,
  fast_forward_speed: 2,
  volume: 100,
  scaleOffset: 0,
  uiTransitions: true,
  backgroundGlowStrength: 80,
  displayGlowStrength: 40,
  glowEnabled: true,
  glowQuality: 4,
  glowRadius: 0.5,
  scanlineStrength: 20,
  scanlineSize: 0.25,
  ambientLight: 70,
};

export type Options = typeof defaultOptions;

export const toEmulatorOptions = (options: Options) => {
  return {
    palette: palettes[paletteNames[options.paletteIndex]],
    volume: options.volume / 100,
    scale_offset: options.scaleOffset,
    background_glow_strength: options.backgroundGlowStrength / 100,
    display_glow_strength: options.displayGlowStrength / 100,
    glow_enabled: options.glowEnabled,
    glow_iterations: options.glowQuality * 2,
    glow_radius: options.glowRadius,
    scanline_strength: options.scanlineStrength / 100,
    scanline_size: options.scanlineSize,
    ambient_light: options.ambientLight / 100,
  } as EmulatorOptions
}

export const loadOptions = () => {
  let loaded = localStorage.getItem("options");
  if (loaded !== null) {
    return JSON.parse(loaded) as Options;
  }
  else {
    return defaultOptions;
  }
}

export const saveOptions = (options: Options) => {
  localStorage.setItem("options", JSON.stringify(options));
}
