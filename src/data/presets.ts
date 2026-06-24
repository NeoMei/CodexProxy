export interface Preset {
  id: string;
  name: string;
  model: string;
  upstream: string;
  contextWindow: number;
  maxOutputTokens: number;
  /** Short label shown in preset selector */
  label: string;
}

export const BUILTIN_PRESETS: Preset[] = [
  // ── 国产 ──
  {
    id: "preset-kimi",
    name: "Kimi K2.7 Code",
    model: "kimi-for-coding",
    upstream: "https://api.kimi.com/coding/v1",
    contextWindow: 262144,
    maxOutputTokens: 32768,
    label: "Kimi (月之暗面)",
  },
  {
    id: "preset-glm",
    name: "GLM 5.2",
    model: "glm-5.2",
    upstream: "https://open.bigmodel.cn/api/anthropic/v1",
    contextWindow: 200000,
    maxOutputTokens: 32768,
    label: "GLM (智谱)",
  },
  {
    id: "preset-deepseek",
    name: "DeepSeek V4 Pro",
    model: "deepseek-v4-pro",
    upstream: "https://api.deepseek.com/anthropic/v1",
    contextWindow: 1000000,
    maxOutputTokens: 384000,
    label: "DeepSeek",
  },
  {
    id: "preset-volcengine",
    name: "Volcengine AgentPlan",
    model: "doubao-seed-2.0",
    upstream: "https://ark.cn-beijing.volces.com/api/anthropic/v1",
    contextWindow: 200000,
    maxOutputTokens: 32768,
    label: "火山方舟 (豆包)",
  },
  {
    id: "preset-bailian",
    name: "Bailian Coding Plan",
    model: "qwen-plus",
    upstream: "https://dashscope.aliyuncs.com/compatible-mode/anthropic/v1",
    contextWindow: 200000,
    maxOutputTokens: 32768,
    label: "阿里百炼 (通义)",
  },
  // ── 国外 ──
  {
    id: "preset-openai-responses",
    name: "OpenAI (Responses)",
    model: "gpt-5.5",
    upstream: "https://api.openai.com/v1",
    contextWindow: 272000,
    maxOutputTokens: 128000,
    label: "OpenAI (GPT-5.5)",
  },
  {
    id: "preset-anthropic",
    name: "Claude Opus 4",
    model: "claude-opus-4-20250514",
    upstream: "https://api.anthropic.com/v1",
    contextWindow: 200000,
    maxOutputTokens: 32768,
    label: "Anthropic (Claude Opus 4)",
  },
  {
    id: "preset-google",
    name: "Gemini 2.5 Pro",
    model: "gemini-2.5-pro",
    upstream: "https://generativelanguage.googleapis.com/v1beta",
    contextWindow: 1048576,
    maxOutputTokens: 65536,
    label: "Google (Gemini 2.5 Pro)",
  },
];

/** Find a preset by model slug */
export function findPreset(model: string): Preset | undefined {
  return BUILTIN_PRESETS.find(p => p.model === model);
}
