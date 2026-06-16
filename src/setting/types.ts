/** 一条快捷键配置 */
export interface ShortcutItem {
    id: string;
    label: string;
    keys: string[];
    defaultKeys: string[];
    save: {
        id: string;
        index: number;
    }[];
}

/** 通用设置项 */
export interface GeneralSetting {
    id: string;
    label: string;
    description: string;
    value: boolean | string | number;
}

/** 真正的配置项，定义在Rust */
export interface RustAppConfig {
    theme: string;
    auto_start: boolean;
    auto_play: boolean;
    pronunciation: string;
    pronunciation_volume: number;
    point_key: string;
    pinned_key: string;
    hide_win_key: string;
    select_text_modifiers: string;
    select_text_code: string;
    search_win_modifiers: string;
    search_win_code: string;
    first_setup_completed: boolean;
}
