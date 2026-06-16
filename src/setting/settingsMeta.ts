import { RustAppConfig } from "./types";

type Keys = keyof Pick<
    RustAppConfig,
    | "select_text_modifiers"
    | "select_text_code"
    | "search_win_modifiers"
    | "search_win_code"
    | "point_key"
    | "pinned_key"
    | "hide_win_key"
>;

export const GENERAL_SETTINGS_TEMPLATE: {
    id: keyof Pick<
        RustAppConfig,
        "auto_start" | "auto_play" | "pronunciation" | "pronunciation_volume"
    >;
    label: string;
    description: string;
}[] = [
    {
        id: "auto_start" as const,
        label: "自动启动",
        description: "应用将在系统启动时自动运行",
    },
    {
        id: "auto_play" as const,
        label: "自动播放",
        description: "查词成功后自动播放单词音频",
    },
    {
        id: "pronunciation" as const,
        label: "单词发音",
        description: "应用美式或英式的发音与英标",
    },
    {
        id: "pronunciation_volume" as const,
        label: "发音音量",
        description: "设置英文单词发音的音量大小",
    },
];

export const SHORTCUT_LIST_TEMPLATE: {
    id: string;
    label: string;
    rustKeys: Keys[];
    defaultKeys: string[];
    save: {
        id: string;
        index: number;
    }[];
}[] = [
    {
        id: "point_key",
        label: "光标翻译",
        rustKeys: ["point_key"],
        defaultKeys: ["F2"],
        save: [
            {
                id: "point_key",
                index: 0,
            },
        ],
    },
    {
        id: "pinned_key",
        label: "窗口置顶",
        rustKeys: ["pinned_key"],
        defaultKeys: ["F1"],
        save: [
            {
                id: "pinned_key",
                index: 0,
            },
        ],
    },
    {
        id: "hide_win_key",
        label: "隐藏窗口",
        rustKeys: ["hide_win_key"],
        defaultKeys: ["Tab"],
        save: [
            {
                id: "hide_win_key",
                index: 0,
            },
        ],
    },
    {
        id: "select_text",
        label: "选词翻译",
        rustKeys: ["select_text_modifiers", "select_text_code"],
        defaultKeys: ["SUPER", "Digit1"],
        save: [
            {
                id: "select_text_modifiers",
                index: 0,
            },
            {
                id: "select_text_code",
                index: 1,
            },
        ],
    },
    {
        id: "search_win",
        label: "唤醒搜索",
        rustKeys: ["search_win_modifiers", "search_win_code"],
        defaultKeys: ["SUPER", "Digit2"],
        save: [
            {
                id: "search_win_modifiers",
                index: 0,
            },
            {
                id: "search_win_code",
                index: 1,
            },
        ],
    },
];

export const MODIFIER_MAP: Record<string, string> = {
    CONTROL: "Ctrl",
    SUPER: "Cmd",
    ALT: "Alt",
    SHIFT: "Shift",
    Digit1: "1",
    Digit2: "2",
    Digit3: "3",
    Digit4: "4",
    Digit5: "5",
    Digit6: "6",
    Digit7: "7",
    Digit8: "8",
    Digit9: "9",
    Digit0: "0",
    KeyA: "A",
    KeyB: "B",
    KeyC: "C",
    KeyD: "D",
    KeyE: "E",
    KeyF: "F",
    KeyG: "G",
    KeyH: "H",
    KeyI: "I",
    KeyJ: "J",
    KeyK: "K",
    KeyL: "L",
    KeyM: "M",
    KeyN: "N",
    KeyO: "O",
    KeyP: "P",
    KeyQ: "Q",
    KeyR: "R",
    KeyS: "S",
    KeyT: "T",
    KeyU: "U",
    KeyV: "V",
    KeyW: "W",
    KeyX: "X",
    KeyY: "Y",
    KeyZ: "Z",
    Escape: "Esc",
    Backspace: "Backspace",
    Delete: "Delete",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Semicolon: ";",
    Quote: "'",
    Backquote: "`",
    Comma: ",",
    Period: ".",
    Slash: "/",
    Backslash: "\\",
};

export const MODIFIER_MAP_RUST: Record<string, string> = {
    ...Object.fromEntries(
        Object.entries(MODIFIER_MAP).map(([key, value]) => [value, key]),
    ),
    Opt: "ALT",
};
