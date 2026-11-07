import { MODIFIER_MAP } from "../settingsMeta";

export function keyLabel(key: string, isMac: boolean): string {
    if (key === "SUPER") return isMac ? "Cmd" : "Win";
    if (key === "ALT") return isMac ? "Opt" : "Alt";
    if (/^Numpad[0-9]$/.test(key)) return `小键盘 ${key.slice(-1)}`;
    return MODIFIER_MAP[key] ?? key;
}

/** 单独渲染一个按键标签 */
function KeyTag({ label }: { label: string }) {
    return (
        <kbd className="inline-flex items-center justify-center min-w-7 h-6 px-1.5 text-[11px] font-semibold rounded-md bg-gray-100 border border-gray-200 text-gray-700 shadow-[inset_0_-1px_0_#d0d0d0]">
            {label}
        </kbd>
    );
}

/** 快捷键组合渲染 */
export function ShortcutKeys({
    keys,
    isMac,
}: {
    keys: string[];
    isMac: boolean;
}) {
    if (keys.length === 0) {
        return (
            <span className="text-[11px] text-tagSecondW italic">未设置</span>
        );
    }
    return (
        <div className="flex flex-wrap justify-end items-center gap-1">
            {keys.map((k, i) => (
                <span key={i} className="flex items-center gap-1">
                    <KeyTag label={keyLabel(k, isMac)} />
                    {i < keys.length - 1 && <span className="text-xs">+</span>}
                </span>
            ))}
        </div>
    );
}
