import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Provider {
  id: string;
  name: string;
  model: string;
  upstream: string;
  api_key: string;
  context_window: number;
  max_output_tokens: number;
  enabled: boolean;
  sort_index: number;
}

function App() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [proxyRunning, setProxyRunning] = useState(false);
  const [proxyPort, setProxyPort] = useState(15731);
  const [codexConfig, setCodexConfig] = useState("");
  const [editing, setEditing] = useState<Provider | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [testResult, setTestResult] = useState<Record<string, string>>({});
  const [testing, setTesting] = useState<Record<string, boolean>>({});
  const [autoStart, setAutoStart] = useState(false);

  const refreshProviders = useCallback(async () => {
    const list = await invoke<Provider[]>("list_providers");
    setProviders(list);
  }, []);

  const refreshStatus = useCallback(async () => {
    const running = await invoke<boolean>("proxy_status");
    setProxyRunning(running);
    try {
      const port = await invoke<number>("proxy_port");
      setProxyPort(port);
    } catch {}
    try {
      const cfg = await invoke<string>("read_codex_config");
      setCodexConfig(cfg);
    } catch {}
  }, []);

  useEffect(() => {
    refreshProviders();
    refreshStatus();
    const timer = setInterval(refreshStatus, 3000);
    return () => clearInterval(timer);
  }, [refreshProviders, refreshStatus]);

  useEffect(() => {
    invoke<string>("get_setting", { key: "auto_start" }).then(v => setAutoStart(v === "true")).catch(() => {});
  }, []);

  const toggleProxy = async () => {
    if (proxyRunning) {
      await invoke("stop_proxy");
    } else {
      await invoke("start_proxy");
    }
    refreshStatus();
  };

  const testProvider = async (p: Provider) => {
    setTesting(t => ({ ...t, [p.id]: true }));
    try {
      const result = await invoke<string>("test_connection", { provider: p });
      setTestResult(t => ({ ...t, [p.id]: "✅ " + result }));
    } catch (e: any) {
      setTestResult(t => ({ ...t, [p.id]: "❌ " + e }));
    }
    setTesting(t => ({ ...t, [p.id]: false }));
  };

  const applyModel = async (model: string) => {
    await invoke("apply_to_codex", { model });
    refreshStatus();
  };

  const saveProvider = async (p: Provider) => {
    if (!p.id) p.id = await invoke<string>("generate_id");
    await invoke("save_provider", { provider: p });
    setEditing(null);
    setShowAdd(false);
    refreshProviders();
  };

  const deleteProvider = async (id: string) => {
    await invoke("delete_provider", { id });
    refreshProviders();
  };

  const toggleAutoStart = async () => {
    const next = !autoStart;
    setAutoStart(next);
    await invoke("set_setting", { key: "auto_start", value: String(next) });
  };

  const emptyProvider = (): Provider => ({
    id: "", name: "", model: "", upstream: "", api_key: "",
    context_window: 262144, max_output_tokens: 32768, enabled: true, sort_index: 0,
  });

  return (
    <div className="h-screen flex flex-col">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-3 border-b border-zinc-800 bg-zinc-950">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">Coding Plan Proxy</h1>
          <span className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs ${
            proxyRunning ? "bg-green-900/50 text-green-400" : "bg-zinc-800 text-zinc-500"
          }`}>
            <span className={`w-2 h-2 rounded-full ${proxyRunning ? "bg-green-400" : "bg-zinc-600"}`} />
            {proxyRunning ? `Running :${proxyPort}` : "Stopped"}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={toggleProxy}
            className={`px-4 py-1.5 rounded text-sm font-medium transition ${
              proxyRunning
                ? "bg-red-600/20 text-red-400 border border-red-600/30 hover:bg-red-600/30"
                : "bg-green-600/20 text-green-400 border border-green-600/30 hover:bg-green-600/30"
            }`}
          >
            {proxyRunning ? "Stop Proxy" : "Start Proxy"}
          </button>
        </div>
      </header>

      {/* Main */}
      <main className="flex-1 overflow-auto p-6">
        {/* Providers */}
        <div className="mb-6">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500">Providers</h2>
            <button
              onClick={() => { setEditing(emptyProvider()); setShowAdd(true); }}
              className="px-3 py-1 text-sm bg-blue-600/20 text-blue-400 border border-blue-600/30 rounded hover:bg-blue-600/30 transition"
            >
              + Add Provider
            </button>
          </div>

          <div className="space-y-2">
            {providers.map(p => (
              <div key={p.id} className={`flex items-center gap-3 p-3 rounded-lg border transition ${
                p.enabled ? "border-zinc-700 bg-zinc-900/50" : "border-zinc-800 bg-zinc-950/50 opacity-60"
              }`}>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-sm">{p.name}</span>
                    <code className="text-xs bg-zinc-800 px-1.5 py-0.5 rounded text-zinc-400">{p.model}</code>
                    {p.enabled && proxyRunning && (
                      <button
                        onClick={() => applyModel(p.model)}
                        className="text-xs px-2 py-0.5 bg-purple-600/20 text-purple-400 border border-purple-600/30 rounded hover:bg-purple-600/30"
                        title="Set as current model in Codex"
                      >
                        Apply to Codex
                      </button>
                    )}
                  </div>
                  <div className="text-xs text-zinc-500 mt-0.5 truncate">{p.upstream}</div>
                  {testResult[p.id] && (
                    <div className={`text-xs mt-1 ${testResult[p.id].startsWith("✅") ? "text-green-400" : "text-red-400"}`}>
                      {testResult[p.id]}
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-1">
                  <button
                    onClick={() => testProvider(p)}
                    disabled={testing[p.id]}
                    className="p-1.5 text-xs text-zinc-400 hover:text-white hover:bg-zinc-700 rounded transition disabled:opacity-50"
                    title="Test connection"
                  >
                    {testing[p.id] ? "⏳" : "🔍"}
                  </button>
                  <button
                    onClick={() => { setEditing({ ...p }); setShowAdd(false); }}
                    className="p-1.5 text-xs text-zinc-400 hover:text-white hover:bg-zinc-700 rounded transition"
                    title="Edit"
                  >
                    ✏️
                  </button>
                  <button
                    onClick={() => deleteProvider(p.id)}
                    className="p-1.5 text-xs text-zinc-400 hover:text-red-400 hover:bg-zinc-700 rounded transition"
                    title="Delete"
                  >
                    🗑
                  </button>
                </div>
              </div>
            ))}
            {providers.length === 0 && (
              <div className="text-center py-8 text-zinc-600 text-sm">
                No providers yet. Click "+ Add Provider" to add one.
              </div>
            )}
          </div>
        </div>

        {/* Settings */}
        <div className="border-t border-zinc-800 pt-6">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500 mb-3">Settings</h2>
          <div className="space-y-3 max-w-md">
            <label className="flex items-center justify-between p-3 rounded-lg border border-zinc-700 bg-zinc-900/50">
              <div>
                <div className="text-sm font-medium">Auto-start proxy</div>
                <div className="text-xs text-zinc-500">Launch proxy on app startup</div>
              </div>
              <button
                onClick={toggleAutoStart}
                className={`w-10 h-5 rounded-full transition relative ${autoStart ? "bg-blue-600" : "bg-zinc-700"}`}
              >
                <span className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition ${autoStart ? "left-5" : "left-0.5"}`} />
              </button>
            </label>
          </div>
        </div>

        {/* Codex Config Preview */}
        {codexConfig && (
          <div className="border-t border-zinc-800 pt-6 mt-6">
            <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500 mb-3">Codex Config (~/.codex/config.toml)</h2>
            <pre className="text-xs bg-zinc-950 border border-zinc-800 rounded-lg p-3 overflow-x-auto text-zinc-400 font-mono">
              {codexConfig}
            </pre>
          </div>
        )}
      </main>

      {/* Modal: Add/Edit Provider */}
      {(editing || showAdd) && (
        <ProviderEditor
          provider={editing ?? emptyProvider()}
          onSave={saveProvider}
          onClose={() => { setEditing(null); setShowAdd(false); }}
        />
      )}
    </div>
  );
}

function ProviderEditor({ provider, onSave, onClose }: {
  provider: Provider;
  onSave: (p: Provider) => void;
  onClose: () => void;
}) {
  const [form, setForm] = useState({ ...provider });

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-zinc-900 border border-zinc-700 rounded-xl p-6 w-full max-w-lg mx-4" onClick={e => e.stopPropagation()}>
        <h3 className="text-lg font-semibold mb-4">{provider.id ? "Edit Provider" : "Add Provider"}</h3>
        <div className="space-y-3">
          <Field label="Display Name" value={form.name} onChange={v => setForm(f => ({ ...f, name: v }))} />
          <Field label="Model Slug" value={form.model} onChange={v => setForm(f => ({ ...f, model: v }))}
            placeholder="e.g. glm-5.2, kimi-for-coding" />
          <Field label="Upstream URL" value={form.upstream} onChange={v => setForm(f => ({ ...f, upstream: v }))}
            placeholder="https://api.kimi.com/coding/v1" />
          <Field label="API Key" value={form.api_key} onChange={v => setForm(f => ({ ...f, api_key: v }))} type="password"
            placeholder="sk-..." />
          <div className="grid grid-cols-2 gap-3">
            <Field label="Context Window" value={String(form.context_window)}
              onChange={v => setForm(f => ({ ...f, context_window: Number(v) || 262144 }))} />
            <Field label="Max Output Tokens" value={String(form.max_output_tokens)}
              onChange={v => setForm(f => ({ ...f, max_output_tokens: Number(v) || 32768 }))} />
          </div>
        </div>
        <div className="flex justify-end gap-2 mt-6">
          <button onClick={onClose} className="px-4 py-2 text-sm text-zinc-400 hover:text-white transition">Cancel</button>
          <button
            onClick={() => onSave(form)}
            className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}

function Field({ label, value, onChange, type = "text", placeholder = "" }: {
  label: string; value: string; onChange: (v: string) => void; type?: string; placeholder?: string;
}) {
  return (
    <label className="block">
      <span className="text-xs text-zinc-500 mb-1 block">{label}</span>
      <input
        type={type}
        value={value}
        onChange={e => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm text-white placeholder:text-zinc-600 focus:outline-none focus:border-blue-600 transition"
      />
    </label>
  );
}

export default App;
