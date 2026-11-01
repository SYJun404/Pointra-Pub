import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { flushSync } from "react-dom";

const WINDOW_WILL_HIDE = "window-will-hide";

export function hideMainWindow() {
    window.dispatchEvent(new Event(WINDOW_WILL_HIDE));
    return invoke<void>("hide_main_window");
}

/**
 * 自定义 Hook：将最新值同步到 Ref 中，避免 useEffect 依赖频繁触发
 */
function useLatest<T>(value: T) {
    const ref = useRef(value);
    ref.current = value;
    return ref;
}

export function useWindowListener(
    isPinned: boolean,
    setIsPinned: (isPinned: boolean) => void,
    hideKey: string,
    pinKey: string,
) {
    // 将所有频繁改变的属性收纳到一个 ref 对象中
    const stateRef = useLatest({ isPinned, hideKey, pinKey });

    useEffect(() => {
        const handleMouseEnter = () => {
            invoke("update_hover_status", { hovered: true });
        };

        const handleMouseLeave = () => {
            if (!stateRef.current.isPinned) {
                invoke("update_hover_status", { hovered: false });
                void hideMainWindow();
            }
        };

        const handleKeydown = (e: KeyboardEvent) => {
            const { hideKey, pinKey, isPinned } = stateRef.current;

            if (e.key === hideKey) {
                void hideMainWindow();
            }
            if (e.key === pinKey) {
                setIsPinned(!isPinned);
            }
        };

        const isMac =
            navigator.platform?.toLowerCase().includes("mac") ?? false;
        const docElment = isMac ? document.body : document;

        // 绑定事件
        docElment.addEventListener("mouseenter", handleMouseEnter);
        docElment.addEventListener("mouseleave", handleMouseLeave);
        document.addEventListener("keydown", handleKeydown);

        // 清理事件
        return () => {
            docElment.removeEventListener("mouseenter", handleMouseEnter);
            docElment.removeEventListener("mouseleave", handleMouseLeave);
            document.removeEventListener("keydown", handleKeydown);
        };
    }, []);
}

export function useOnWindowChange(onHide: () => void) {
    const savedOnHide = useLatest(onHide);

    useEffect(() => {
        let disposed = false;
        const handleHide = () => {
            if (disposed) return;
            // 隐藏 WebView 前提交初始界面，避免恢复时先显示旧内容。
            flushSync(() => savedOnHide.current());
        };

        window.addEventListener(WINDOW_WILL_HIDE, handleHide);
        const listenerPromise = getCurrentWindow().listen(
            WINDOW_WILL_HIDE,
            handleHide,
        );

        return () => {
            disposed = true;
            window.removeEventListener(WINDOW_WILL_HIDE, handleHide);
            listenerPromise.then((unlisten) => unlisten());
        };
    }, []);
}
