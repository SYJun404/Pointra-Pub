import { Xmark, ArrowsRotateLeft, Pencil } from "@gravity-ui/icons";
import type { ShortcutItem } from "../types";
import { keyLabel, ShortcutKeys } from "./KeyTag";
import type { UseShortcutManagerReturn } from "../hooks";
import { useEffect } from "react";
import { isSingleKeyShortcut, shortcutSignature } from "../utils";

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
            onKeyDown(e, recordingId);
        };

        window.addEventListener("keydown", handler);

        return () => {
            window.removeEventListener("keydown", handler);
        };
    }, [recordingId, onKeyDown]);

    const showTips = (id: string) => {
        if (isSingleKeyShortcut(id)) {
            return "支持 Esc、Tab 或 F1–F12";
        }
        const modifiers = ["SUPER", "ALT", "CONTROL", "SHIFT"]
            .map((key) => keyLabel(key, isMac))
            .join(" / ");
        return `按住修饰键（${modifiers}），再按主键`;
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
                    const changed =
                        shortcutSignature(item.keys) !==
                        shortcutSignature(item.defaultKeys);
                    const defaultLabel = item.defaultKeys
                        .map((key) => keyLabel(key, isMac))
                        .join(" + ");

                    return (
                        <div
                            key={item.id}
                            className={`transition-colors ${
                                isRec ? "bg-blueBgW" : "hover:bg-subBgW"
                            }`}
                        >
                            <div className="flex items-center justify-between p-3 min-h-13">
                                {/* 左侧 */}
                                <div className="flex-1 min-w-20 mr-3">
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
                                    {item.description && (
                                        <p className="text-[11px] text-tagSecondW mt-0.5">
                                            {item.description}
                                        </p>
                                    )}
                                </div>

                                {/* 右侧 */}
                                <div className="flex items-center justify-end min-w-0">
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
                                            <button
                                                type="button"
                                                aria-label={`取消录制${item.label}`}
                                                className="text-mainBlueW cursor-pointer hover:opacity-70"
                                                onClick={onCancelRecording}
                                            >
                                                <Xmark width={14} height={14} />
                                            </button>
                                        </div>
                                    ) : (
                                        /* ---- 显示 ---- */
                                        <div className="flex flex-wrap items-center justify-end gap-2">
                                            <ShortcutKeys
                                                keys={item.keys}
                                                isMac={isMac}
                                            />
                                            <div className="flex shrink-0 items-center gap-1">
                                                {changed && (
                                                    <button
                                                        type="button"
                                                        title={`默认快捷键 ${defaultLabel}`}
                                                        aria-label={`重置${item.label}为 ${defaultLabel}`}
                                                        onClick={() =>
                                                            onResetOne(item.id)
                                                        }
                                                        className="flex items-center gap-1 px-1.5 h-6 rounded-md border border-borderMainW
                                                                   bg-white text-tagSecondW text-[11px] hover:bg-subBgW hover:text-mainTitleW
                                                                   cursor-pointer transition-colors"
                                                    >
                                                        <ArrowsRotateLeft
                                                            width={12}
                                                            height={12}
                                                        />
                                                        重置
                                                    </button>
                                                )}
                                                <button
                                                    type="button"
                                                    aria-label={`编辑${item.label}`}
                                                    title={`编辑${item.label}`}
                                                    onClick={() =>
                                                        onStartRecording(
                                                            item.id,
                                                        )
                                                    }
                                                    className="flex items-center justify-center w-6 h-6 rounded-md border border-borderMainW
                                                               bg-white text-tagSecondW hover:bg-blueBgW hover:text-mainBlueW hover:border-blueBorderW
                                                               transition-all active:scale-90 cursor-pointer"
                                                >
                                                    <Pencil
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
                                <div className="px-3 pb-2.5">
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
