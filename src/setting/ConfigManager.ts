// ConfigManager.ts
import { invoke } from "@tauri-apps/api/core";
import { RustAppConfig, GeneralSetting, ShortcutItem } from "./types";
import {
    GENERAL_SETTINGS_TEMPLATE,
    SHORTCUT_LIST_TEMPLATE,
    MODIFIER_MAP_RUST,
} from "./settingsMeta.ts";

export class ConfigManager {
    /**
     * 从后端获取原始的 RustAppConfig
     */
    static async fetchRawConfig(): Promise<RustAppConfig> {
        return await invoke<RustAppConfig>("get_config");
    }

    static async stopShortcuts(): Promise<void> {
        await invoke<void>("stop_shortcuts");
    }

    /**
     * 获取前端渲染所需的完整打包数据
     */
    static async getAllSettings() {
        const rustConfig = await this.fetchRawConfig();

        const general = GENERAL_SETTINGS_TEMPLATE.map((meta) => ({
            ...meta,
            value: rustConfig[meta.id],
        }));

        const shortcuts: ShortcutItem[] = SHORTCUT_LIST_TEMPLATE.map(
            (meta) => ({
                ...meta,
                keys: meta.rustKeys.map((key) => rustConfig[key]),
            }),
        );

        const first_setup_completed = rustConfig.first_setup_completed;

        return { general, shortcuts, first_setup_completed };
    }

    /**
     * 更新所有设置项并同步到后端
     */
    static async updateAllSetting(
        shortcuts: ShortcutItem[],
        general: GeneralSetting[],
    ): Promise<boolean> {
        try {
            // 将 shortcuts 和 general 转换为 RustAppConfig 格式
            const config: RustAppConfig = {
                auto_start: false,
                auto_play: false,
                pronunciation: "us",
                pronunciation_volume: 50,
                hide_win_key: "Tab",
                pinned_key: "F1",
                point_key: "F2",
                search_win_code: "Digit2",
                search_win_modifiers: "SUPER",
                select_text_code: "Digit1",
                select_text_modifiers: "SUPER",
                theme: "light",
                first_setup_completed: false,
            };

            for (const setting of general) {
                Object.assign(config, { [setting.id]: setting.value });
            }
            for (const setting of shortcuts) {
                for (const key of setting.save) {
                    Object.assign(config, {
                        [key.id]:
                            MODIFIER_MAP_RUST[setting.keys[key.index]] ??
                            setting.keys[key.index],
                    });
                }
            }

            return await invoke<boolean>("update_config", {
                newConfig: config, //小了A阿斯顿笑了
            });
        } catch (error) {
            console.error(error);
            return false;
        }
    }
}
