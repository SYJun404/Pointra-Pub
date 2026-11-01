import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type CacheStats = {
    entries: number;
    disk_bytes: number;
    memory_bytes: number;
    memory_limit: number;
    disk_limit: number;
};
const mb = (bytes: number) => (bytes / 1024 / 1024).toFixed(2);

export default function CacheSection() {
    const [stats, setStats] = useState<CacheStats | null>(null);
    const [busy, setBusy] = useState(false);
    const [confirmClear, setConfirmClear] = useState(false);
    const [message, setMessage] = useState("");
    const running = useRef(false);
    const refresh = async () =>
        setStats(await invoke<CacheStats>("cache_stats"));
    useEffect(() => {
        void refresh().catch((error) => setMessage(String(error)));
    }, []);

    const run = async (action: "import" | "export" | "clear") => {
        if (running.current) return;
        running.current = true;
        setBusy(true);
        setMessage("");
        setConfirmClear(false);
        try {
            if (action === "import") {
                const count = await invoke<number | null>("import_word_cache");
                if (count !== null)
                    setMessage(
                        `已合并 ${count} 条记录，超出容量的旧记录会自动清理。`,
                    );
            } else if (action === "export") {
                if (await invoke<boolean>("export_word_cache"))
                    setMessage("缓存已导出，可在其他设备导入。");
            } else {
                await invoke("clear_word_cache");
                setMessage("缓存已清空。");
            }
            await refresh();
        } catch (error) {
            setMessage(`操作失败：${String(error)}`);
        } finally {
            running.current = false;
            setBusy(false);
        }
    };
    const buttonClass =
        "text-xs px-3 py-1.5 rounded-lg border border-borderMainW text-mainBlueW cursor-pointer transition-transform active:scale-90 disabled:opacity-40 disabled:cursor-default";
    return (
        <section>
            <div className="flex items-center gap-2 mb-3">
                <div className="w-1 h-4 rounded-full bg-mainBlueW" />
                <h2 className="text-sm font-medium text-mainTitleW">
                    单词缓存
                </h2>
            </div>
            <div className="rounded-xl border border-borderMainW bg-white p-3 space-y-3">
                <div className="text-xs text-mainTitleW space-y-1">
                    <p>
                        自动保存查询过的单词，已缓存：
                        {stats ? `${stats.entries} 条` : "—"}
                    </p>
                    <div className="flex gap-6">
                        <p>
                            内存：
                            {stats
                                ? `${mb(stats.memory_bytes)} / ${mb(stats.memory_limit)} MB`
                                : "—"}
                        </p>
                        <p>
                            磁盘：
                            {stats
                                ? `${mb(stats.disk_bytes)} / ${mb(stats.disk_limit)} MB`
                                : "—"}
                        </p>
                    </div>
                </div>
                <div className="flex justify-between">
                    <button
                        className={buttonClass}
                        disabled={busy}
                        onClick={() => void run("export")}
                    >
                        导出缓存
                    </button>
                    <button
                        className={buttonClass}
                        disabled={busy}
                        onClick={() => void run("import")}
                    >
                        导入缓存
                    </button>
                    <button
                        className={buttonClass}
                        disabled={busy}
                        onClick={() => setConfirmClear(true)}
                    >
                        清空缓存
                    </button>
                    <button
                        className={buttonClass}
                        disabled={busy}
                        onClick={() =>
                            void refresh().catch((e) => setMessage(String(e)))
                        }
                    >
                        刷新统计
                    </button>
                </div>
                {confirmClear && (
                    <div className="flex items-center gap-3 text-xs text-tagW">
                        <span>确定删除全部缓存？</span>
                        <button
                            className={buttonClass}
                            onClick={() => void run("clear")}
                        >
                            确定
                        </button>
                        <button
                            className={buttonClass}
                            onClick={() => setConfirmClear(false)}
                        >
                            取消
                        </button>
                    </div>
                )}
                {busy && (
                    <p
                        className="text-[11px] text-center text-tagW"
                        role="status"
                        aria-live="polite"
                    >
                        {busy ? "正在处理…" : message}
                    </p>
                )}
            </div>
        </section>
    );
}
