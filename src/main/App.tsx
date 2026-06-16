import "../assets/css/App.css";
import { useWindowListener } from "./utils/useCustom";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { Routes, useNavigate, Route } from "react-router-dom";
import useUiStore from "./store/useUiStore";
import SearchPage from "./pages/SearchPage";
import HomePage from "./pages/HomePage";
import useConfigStore from "./store/useConfigStore";
import { invoke } from "@tauri-apps/api/core";
import { RustAppConfig } from "../setting/types";
import { useShallow } from "zustand/react/shallow";
import { usePlatform } from "../hooks/usePlatform";

function App() {
    const navigate = useNavigate();
    const { isMac } = usePlatform();

    const isPinned = useUiStore((state) => state.isPinned);
    const setIsPinned = useUiStore((state) => state.setIsPinned);
    const setUpdateUrl = useUiStore((state) => state.setUpdateUrl);

    const { isAutoPlay, pronunciation, pinnedKey, hideWinKey } = useConfigStore(
        useShallow((state) => ({
            isAutoPlay: state.auto_play,
            pronunciation: state.pronunciation,
            pinnedKey: state.pinned_key,
            hideWinKey: state.hide_win_key,
        })),
    );

    const updateConfig = useConfigStore((state) => state.update);

    // 监听鼠标移入/出窗口
    useWindowListener(isPinned, setIsPinned, hideWinKey, pinnedKey);

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<{ type: string; data: any }>(
                "win-event",
                (event) => {
                    if (event.payload.type === "search") {
                        navigate("/search");
                    } else if (event.payload.type === "config") {
                        updateConfig(event.payload.data);
                    }
                },
            );

            // 返回清理函数
            return unlisten;
        };

        const listenerPromise = setupListener();

        // 初始化配置
        invoke<RustAppConfig>("get_config").then((config) => {
            updateConfig(config);
        });

        // 检测更新
        invoke<any>("trigger_update_check", { version: __APP_VERSION__ }).then(
            (data) => {
                setUpdateUrl(data.url);
            },
        );

        return () => {
            listenerPromise.then((unlisten) => unlisten());
        };
    }, []);
    return (
        <Routes>
            <Route
                path="/"
                element={
                    <HomePage
                        isAutoPlay={isAutoPlay}
                        isUs={pronunciation === "us"}
                    />
                }
            />
            <Route path="/search" element={<SearchPage isMac={isMac} />} />
        </Routes>
    );
}

export default App;
