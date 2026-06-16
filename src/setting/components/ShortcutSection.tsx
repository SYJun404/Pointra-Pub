import { Xmark, ArrowsRotateLeft } from "@gravity-ui/icons";
import type { ShortcutItem } from "../types";
import { ShortcutKeys } from "./KeyTag";
import type { UseShortcutManagerReturn } from "../hooks";
import { useEffect } from "react";

interface ShortcutSectionProps {
    shortcuts: ShortcutItem[];
    recordingId: UseShortcutManagerReturn["recordingId"];
    recordingRef: UseShortcutManagerReturn["recordingRef"];
    conflictIds: UseShortcutManagerReturn["conflictIds"];
    onStartRecording: UseShortcutManagerReturn["startRecording"];
    onCancelRecording: UseShortcutManagerReturn["cancelRecording"];
    onKeyDown: UseShortcutManagerReturn["handleKeyDown"];
    onResetOne: UseShortcutManagerReturn["resetToDefault"];
    isMac: boolean;
}

export default function ShortcutSection({
    shortcuts,
    recordingId,
    recordingRef,
    conflictIds,
    onStartRecording,
    onCancelRecording,
    onKeyDown,
    onResetOne,
    isMac,
}: ShortcutSectionProps) {
    useEffect(() => {
        if (!recordingId) return;

        const handler = (e: KeyboardEvent) => {
            onKeyDown(e as any, recordingId);
        };

        window.addEventListener("keydown", handler);

        return () => {
            window.removeEventListener("keydown", handler);
        };
    }, [recordingId]);

    const isModified = (item: ShortcutItem) =>
        item.keys.join("+") !== item.defaultKeys.join("+");

    const showTips = (id: string) => {
        const altOrOpt = isMac ? "Opt" : "Alt";
        const oneKeyListId = ["point_key", "pinned_key", "hide_win_key"];
        if (oneKeyListId.includes(id)) {
            return (
                `支持按键：Esc / Tab / F1~F12` +
                (id === "point_key"
                    ? "   （ 当前快捷键修改后，重启软件才可生效 ）"
                    : "")
            );
        }
        return `同时按下 Cmd / ${altOrOpt} / Ctrl / Shift +任意键`;
    };

    return (
        <section>
            <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                    <div className="w-1 h-4 rounded-full bg-mainBlueW" />
                    <h2 className="text-sm font-medium text-mainTitleW">
                        快捷按键
                    </h2>
                </div>
                <p className="text-[11px] text-tagSecondW mt-px rounded-md ">
                    Shortcut
                </p>
            </div>

            <div className="rounded-xl border border-borderMainW divide-y divide-borderSubW bg-white overflow-hidden">
                {shortcuts.map((item) => {
                    const isRec = recordingId === item.id;
                    const hasConflict = conflictIds.has(item.id);
                    const changed = isModified(item);

                    return (
                        <div
                            key={item.id}
                            className={`transition-colors ${
                                isRec ? "bg-blueBgW" : "hover:bg-subBgW"
                            }`}
                        >
                            <div className="flex items-center justify-between p-3 min-h-13">
                                {/* 左侧 */}
                                <div className="flex-1 min-w-0 mr-3">
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm text-mainTitleW">
                                            {item.label}
                                        </span>
                                        {hasConflict && (
                                            <span className="text-[10px] text-red-500 bg-red-50 border border-red-200 px-1.5 py-0.5 rounded">
                                                冲突
                                            </span>
                                        )}
                                    </div>
                                </div>

                                {/* 右侧 */}
                                <div className="flex items-center gap-2 shrink-0">
                                    {isRec ? (
                                        /* ---- 录制 ---- */
                                        <div
                                            ref={recordingRef}
                                            className="flex items-center gap-2 px-3 py-1.5 rounded-lg border-2 border-mainBlueW bg-blueBgW"
                                            tabIndex={0}
                                            autoFocus
                                        >
                                            <span className="text-xs text-mainBlueW font-medium animate-pulse">
                                                按下快捷键...
                                            </span>
                                            <Xmark
                                                width={14}
                                                height={14}
                                                color="#4a90d9"
                                                className="cursor-pointer hover:opacity-70"
                                                onClick={onCancelRecording}
                                            />
                                        </div>
                                    ) : (
                                        /* ---- 显示 ---- */
                                        <div className="flex items-center gap-2">
                                            <ShortcutKeys keys={item.keys} />
                                            <div className="flex items-center gap-1">
                                                {changed && (
                                                    <button
                                                        onClick={() =>
                                                            onResetOne(item.id)
                                                        }
                                                        className="flex items-center justify-center w-6 h-6 rounded-md border border-borderMainW
                                                                       bg-white text-tagSecondW hover:bg-amber-50 hover:text-amber-600 hover:border-amber-200
                                                                       transition-all active:scale-90 cursor-pointer"
                                                    >
                                                        <ArrowsRotateLeft
                                                            width={12}
                                                            height={12}
                                                        />
                                                    </button>
                                                )}
                                                <button
                                                    onClick={() =>
                                                        onStartRecording(
                                                            item.id,
                                                        )
                                                    }
                                                    className="flex items-center justify-center w-6 h-6 rounded-md border border-borderMainW
                                                               bg-white text-tagSecondW hover:bg-blueBgW hover:text-mainBlueW hover:border-blueBorderW
                                                               transition-all active:scale-90 cursor-pointer"
                                                >
                                                    <Xmark
                                                        width={12}
                                                        height={12}
                                                    />
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>

                            {isRec && (
                                <div className="px-4 pb-2.5">
                                    <p className="text-[10px] text-mainBlueW/70">
                                        {showTips(item.id)}
                                    </p>
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        </section>
    );
}
