export interface Preset {
  id: string;
  name: string;
  model: string;
  upstream: string;
  contextWindow: number;
  maxOutputTokens: number;
  label: string;
}

import presets from "./presets.json";

export const BUILTIN_PRESETS: Preset[] = presets;

export function findPreset(model: string): Preset | undefined {
  return BUILTIN_PRESETS.find(p => p.model === model);
}
