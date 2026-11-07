import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cursorPosition, getCurrentWindow } from "@tauri-apps/api/window";
import { flushSync } from "react-dom";

const WINDOW_WILL_HIDE = "window-will-hide";

export function isImeKey(event: KeyboardEvent) {
    // Chromium may report 229 for the key that commits a composition.
    return event.isComposing || event.keyCode === 229;
}

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
    windowsSearchKey: string | null,
) {
    // 将所有频繁改变的属性收纳到一个 ref 对象中
    const stateRef = useLatest({ isPinned, hideKey, pinKey, windowsSearchKey });

    useEffect(() => {
        let mouseEvent = 0;
        let composingTarget: EventTarget | null = null;
        const isComposing = () =>
            composingTarget instanceof Node && composingTarget.isConnected;
        const handleCompositionStart = (event: CompositionEvent) => {
            composingTarget = event.target;
        };
        const handleCompositionEnd = () => {
            composingTarget = null;
        };

        const handleMouseEnter = () => {
            mouseEvent += 1;
            invoke("update_hover_status", { hovered: true });
        };

        const handleMouseLeave = async () => {
            const event = ++mouseEvent;
            const { isPinned, windowsSearchKey } = stateRef.current;
            if (isPinned) return;

            if (windowsSearchKey !== null) {
                if (isComposing()) return;
                // An IME popup can take the pointer away from WebView2 without
                // the cursor moving beyond the window's physical bounds.
                const appWindow = getCurrentWindow();
                try {
                    const [cursor, origin, size] = await Promise.all([
                        cursorPosition(),
                        appWindow.innerPosition(),
                        appWindow.innerSize(),
                    ]);
                    if (
                        cursor.x >= origin.x &&
                        cursor.x < origin.x + size.width &&
                        cursor.y >= origin.y &&
                        cursor.y < origin.y + size.height
                    ) {
                        return;
                    }
                } catch (error) {
                    console.error("Failed to verify cursor position", error);
                    return;
                }
                if (
                    event !== mouseEvent ||
                    stateRef.current.windowsSearchKey !== windowsSearchKey ||
                    stateRef.current.isPinned ||
                    isComposing()
                ) {
                    return;
                }
            }
            void invoke("update_hover_status", { hovered: false });
            void hideMainWindow();
        };

        const handleKeydown = (e: KeyboardEvent) => {
            if (isImeKey(e)) return;
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
        document.addEventListener("compositionstart", handleCompositionStart);
        document.addEventListener("compositionend", handleCompositionEnd);

        // 清理事件
        return () => {
            mouseEvent += 1;
            docElment.removeEventListener("mouseenter", handleMouseEnter);
            docElment.removeEventListener("mouseleave", handleMouseLeave);
            document.removeEventListener("keydown", handleKeydown);
            document.removeEventListener(
                "compositionstart",
                handleCompositionStart,
            );
            document.removeEventListener(
                "compositionend",
                handleCompositionEnd,
            );
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
