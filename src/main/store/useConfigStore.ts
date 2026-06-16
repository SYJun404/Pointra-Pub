import { create } from "zustand";
import { RustAppConfig } from "../../setting/types";

interface ConfigFields {
    theme: string;
    auto_play: boolean;
    pronunciation: string;
    pinned_key: string;
    hide_win_key: string;
}

interface ConfigState extends ConfigFields {
    update: (config: RustAppConfig) => void;
}

const initialFields: ConfigFields = {
    theme: "light",
    auto_play: false,
    pronunciation: "us",
    pinned_key: "F1",
    hide_win_key: "Tab",
};

// 核心优化：定义一个安全的提取函数，只拿走 Store 声明过的字段，自动过滤掉多余字段
const extractValidConfig = (
    config: Partial<RustAppConfig>,
): Partial<ConfigFields> => {
    const keys = Object.keys(initialFields) as Array<keyof ConfigFields>;
    const cleanConfig: Partial<ConfigFields> = {};

    for (const key of keys) {
        if (config[key] !== undefined) {
            cleanConfig[key] = config[key] as any;
        }
    }
    return cleanConfig;
};

const useConfigStore = create<ConfigState>()((set) => ({
    ...initialFields,
    update: (config: RustAppConfig) => {
        set(extractValidConfig(config));
    },
}));

export default useConfigStore;
