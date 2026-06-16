import { useState, useEffect, useRef, useCallback } from "react";
import type { ShortcutItem, GeneralSetting } from "./types";
import { eventToKeys, getConflictIds, normalizeKeys } from "./utils";
import { ConfigManager } from "./ConfigManager.ts";
import { toast, ToastContentValue } from "@heroui/react";
import { usePlatform } from "../hooks/usePlatform.ts";
import { getCurrentWindow } from "@tauri-apps/api/window";

export interface UseShortcutManagerReturn {
    shortcuts: ShortcutItem[];
    generalSettings: GeneralSetting[];
    hasChanges: boolean;
    isMac: boolean;
    isFirstSetup: boolean;
    recordingId: string | null;
    recordingRef: React.RefObject<HTMLDivElement | null>;
    conflictIds: Set<string>;
    showToast: (msg: string) => void;
    toggleGeneral: (id: string) => void;
    updateGeneralSetting: (
        id: string,
        value: boolean | string | number,
    ) => void;
    startRecording: (id: string) => void;
    cancelRecording: () => void;
    handleKeyDown: (e: React.KeyboardEvent, id: string) => void;
    resetToDefault: (id: string) => void;
    handleSave: () => void;
    closeApp: () => void;
}

export default function useShortcutManager(): UseShortcutManagerReturn {
    const [shortcuts, setShortcuts] = useState<ShortcutItem[]>([]);
    const [generalSettings, setGeneralSettings] = useState<GeneralSetting[]>(
        [],
    );
    const [recordingId, setRecordingId] = useState<string | null>(null);
    const [hasChanges, setHasChanges] = useState(false);
    const [isFirstSetup, setIsFirstSetup] = useState(false);
    const recordingRef = useRef<HTMLDivElement>(null);
    const { isMac } = usePlatform();
    const initConfig = useRef<null | {
        shortcuts: ShortcutItem[];
        generalSettings: GeneralSetting[];
    }>(null);
    const isRecorded = useRef(false);

    /* ---- Toast ---- */
    const showToast = useCallback(
        (
            msg: string,
            variant?: ToastContentValue["variant"],
            timeout?: number,
        ) => {
            toast(msg, {
                timeout: timeout ?? 2000,
                variant: variant ?? "success",
            });
        },
        [],
    );

    /* ---- 通用设置 单选框---- */
    const toggleGeneral = (id: string) => {
        setGeneralSettings((prev) =>
            prev.map((s) => (s.id === id ? { ...s, value: !s.value } : s)),
        );
        setHasChanges(true);
    };

    const updateGeneralSetting = (
        id: string,
        value: boolean | string | number,
    ) => {
        setGeneralSettings((prev) =>
            prev.map((s) => (s.id === id ? { ...s, value } : s)),
        );
        setHasChanges(true);
    };

    /* ---- 快捷键录制 ---- */
    const startRecording = (id: string) => {
        if (!isRecorded.current) {
            isRecorded.current = true;
            ConfigManager.stopShortcuts();
        }
        setRecordingId(id);
    };

    const cancelRecording = () => {
        ConfigManager.updateAllSetting(shortcuts, generalSettings).then(
            (isSuccess) => {
                if (!isSuccess) {
                    showToast("保存失败, 已恢复初始状态", "danger", 3000);
                    if (initConfig.current) {
                        setShortcuts(initConfig.current.shortcuts);
                        setGeneralSettings(initConfig.current.generalSettings);
                    }
                }
            },
        );
        isRecorded.current = false;
        setRecordingId(null);
    };

    const handleKeyDown = (e: React.KeyboardEvent, id: string) => {
        e.preventDefault();
        e.stopPropagation();
        const oneKeyListId = ["point_key", "pinned_key", "hide_win_key"];

        // 防止长按重复触发
        if (e.repeat) return;

        const isModifierOnly =
            e.key === "Control" ||
            e.key === "Shift" ||
            e.key === "Alt" ||
            e.key === "Meta";

        // 只按修饰键不记录
        if (isModifierOnly && !oneKeyListId.includes(id)) return;

        const keys = eventToKeys(e, isMac); // 建议内部用 e.code + modifier
        console.log(keys);

        if (!keys || keys.length === 0) return;

        // 限制规则,单个按键时
        if (oneKeyListId.includes(id)) {
            const isFunctionKey =
                e.code.startsWith("F") && /^F\d{1,2}$/.test(e.code);

            const special = e.key === "Tab" || e.key === "Escape";
            if (!isFunctionKey && !special) {
                showToast("请仔细查看说明，并使用规定按键", "warning", 4000);
                return;
            }
        }

        // 标准化 key（避免顺序问题）
        const normalized = normalizeKeys(keys);

        const conflict = shortcuts.find((s) => {
            if (s.id === id) return false;

            const a = normalizeKeys(s.keys);

            return (
                a.length === normalized.length &&
                a.every((k) => normalized.includes(k))
            );
        });

        setShortcuts((prev) =>
            prev.map((s) => (s.id === id ? { ...s, keys: normalized } : s)),
        );

        setRecordingId(null);
        setHasChanges(true);

        if (conflict) {
            showToast(`该快捷键与「${conflict.label}」冲突`, "danger");
        }
    };

    const resetToDefault = (id: string) => {
        setShortcuts((prev) =>
            prev.map((s) =>
                s.id === id ? { ...s, keys: [...s.defaultKeys] } : s,
            ),
        );
        setHasChanges(true);
        showToast("已恢复默认");
    };

    const handleSave = async () => {
        setHasChanges(false);
        const success = await ConfigManager.updateAllSetting(
            shortcuts,
            generalSettings,
        );

        if (success) {
            isRecorded.current = false;
            showToast("保存设置成功", "success");
        } else {
            showToast("保存失败, 已恢复初始状态", "danger", 3000);
            if (initConfig.current) {
                setShortcuts(initConfig.current.shortcuts);
                setGeneralSettings(initConfig.current.generalSettings);
            }
        }
    };

    const closeApp = async () => {
        if (isRecorded.current) {
            await ConfigManager.updateAllSetting(shortcuts, generalSettings);
        }
        getCurrentWindow().close();
    };

    /* 点击外部停止录制 */
    useEffect(() => {
        if (recordingId === null) return;
        const handler = (e: MouseEvent) => {
            if (
                recordingRef.current &&
                !recordingRef.current.contains(e.target as Node)
            ) {
                setRecordingId(null);
            }
        };
        const timer = setTimeout(() => {
            document.addEventListener("click", handler);
        }, 0);
        return () => {
            clearTimeout(timer);
            document.removeEventListener("click", handler);
        };
    }, [recordingId]);

    // 初始化加载配置
    useEffect(() => {
        ConfigManager.getAllSettings()
            .then(({ general, shortcuts, first_setup_completed }) => {
                setGeneralSettings(general);
                setShortcuts(shortcuts);
                initConfig.current = { shortcuts, generalSettings };

                if (!first_setup_completed && isMac) {
                    setIsFirstSetup(true);
                }
            })
            .catch((err) => console.error("加载配置失败:", err));
    }, []);

    const conflictIds = getConflictIds(shortcuts);

    return {
        shortcuts,
        generalSettings,
        hasChanges,
        isMac,
        isFirstSetup,
        recordingId,
        recordingRef,
        conflictIds,
        showToast,
        toggleGeneral,
        updateGeneralSetting,
        startRecording,
        cancelRecording,
        handleKeyDown,
        resetToDefault,
        handleSave,
        closeApp,
    };
}
