import { waitForCachePresentation } from "../utils/translationPresentation";
import { useOnWindowChange } from "../utils/useCustom";
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useRef } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Header from "../components/Header";
import Content from "../components/Content";
import Footer from "../components/Footer";
import { TransResultTypes } from "../types/transResult";
import ApiError from "../components/ApiError";
import { judgeSentence } from "../utils/tool";
import CustomToast from "../components/CustomToast";
import "../../assets/css/main/index.css";

function HomePage({
    isAutoPlay,
    isUs,
}: {
    isAutoPlay: boolean;
    isUs: boolean;
}) {
    const navigate = useNavigate();
    const location = useLocation();

    const [result, setResult] = useState<TransResultTypes | null>(null);
    const [error, setError] = useState<string | null>(null);

    const [feedbackTitle, setFeedbackTitle] = useState("出错了");
    const [lastFeedback, setLastFeedback] = useState({
        title: "出错了",
        message: "",
    });
    const [feedbackVersion, setFeedbackVersion] = useState(0);
    const requestSequence = useRef(0);

    // Preserve the outgoing message until its fade-out completes.
    useEffect(() => {
        if (error !== null) {
            setLastFeedback({ title: feedbackTitle, message: error });
        }
    }, [error, feedbackTitle]);

    // 窗口隐藏时执行的回调
    useOnWindowChange(() => {
        requestSequence.current += 1;
        setResult(null);
        setError(null);
        setFeedbackTitle("出错了");
        setLastFeedback({ title: "出错了", message: "" });
        // Reset hidden panels so the next opening does not fade out an old error.
        setFeedbackVersion((version) => version + 1);
    });

    const fetchTranslate = async (word: string) => {
        const sequence = ++requestSequence.current;
        const loadingStartedAt = performance.now();
        setFeedbackTitle("出错了");
        setResult(null);
        setError(null);
        try {
            const res = await invoke<TransResultTypes>("fetch_trans_res", {
                word,
            });
            if (sequence !== requestSequence.current) return;
            await waitForCachePresentation(res, loadingStartedAt);
            if (sequence !== requestSequence.current) return;
            if (res.status === 200) {
                if (judgeSentence(res.data.translate.text)) {
                    navigate("/search", { state: { data: res } });
                } else {
                    setResult(res);
                }
            } else {
                setError(res.msg);
            }
        } catch {
            if (sequence === requestSequence.current) {
                setError("翻译请求失败，请重试");
            }
        }
    };

    useEffect(
        () => () => {
            requestSequence.current += 1;
        },
        [],
    );

    useEffect(() => {
        const cursor = location.state?.cursor;
        if (cursor) {
            requestSequence.current += 1;
            setResult(null);
            setError(null);
            setFeedbackTitle("出错了");
            if (cursor.status === "word") {
                void fetchTranslate(cursor.word);
            } else if (cursor.status === "empty") {
                setFeedbackTitle("未识别到单词");
                setError("请将光标移到文字上，再试一次。");
            } else if (cursor.status === "error") {
                setFeedbackTitle("取词失败");
                setError(cursor.message);
            }
        } else if (location.state?.word) {
            void fetchTranslate(location.state.word);
        }
    }, [location.key]);

    return (
        <div className="pt-3 flex flex-col gap-3 h-screen overflow-hidden">
            <CustomToast />
            <Header />

            <div key={feedbackVersion} className="cursor-feedback">
                <div
                    className={`cursor-feedback-panel ${error === null ? "is-active" : ""}`}
                    aria-hidden={error !== null}
                    inert={error !== null}
                >
                    <Content
                        transResult={error === null ? result : null}
                        isAutoPlay={isAutoPlay}
                        isUs={isUs}
                    />
                </div>
                <div
                    className={`cursor-feedback-panel ${error !== null ? "is-active" : ""}`}
                    aria-hidden={error === null}
                    inert={error === null}
                >
                    <ApiError
                        message={error ?? lastFeedback.message}
                        title={
                            error !== null ? feedbackTitle : lastFeedback.title
                        }
                    />
                </div>
            </div>

            <Footer path={"home"} />
        </div>
    );
}

export default HomePage;
