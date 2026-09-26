import { useOnWindowChange } from "../utils/useCustom";
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
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

    // 窗口隐藏时执行的回调
    useOnWindowChange(() => {
        setResult(null);
        setError(null);
    });

    const fetchTranslate = async (word: string) => {
        const res = await invoke<TransResultTypes>("fetch_trans_res", {
            word: word,
        });
        if (res.status === 200) {
            if (judgeSentence(res.data.translate.text)) {
                navigate("/search", { state: { data: res } });
            } else {
                setResult(res);
            }
        } else {
            setError(res.msg);
        }
    };

    // 取词事件统一由 App 层接收后通过路由 state 传进来，
    // 这里响应 state 变化而不是只在挂载时读一次：
    // 这样窗口已经停在首页时连续取词也能刷新结果。
    useEffect(() => {
        const word = location.state?.word;
        if (!word) return;

        const source = location.state?.source ?? "";
        console.debug(
            `[Pointra] translate word=${JSON.stringify(word)} source=${source || "unknown"}`,
        );
        fetchTranslate(word);
    }, [location.state]);

    return (
        <div className="pt-3 flex flex-col gap-3 h-screen overflow-hidden">
            <CustomToast />
            <Header />

            {error === null ? (
                <Content
                    transResult={result}
                    isAutoPlay={isAutoPlay}
                    isUs={isUs}
                />
            ) : (
                <ApiError message={error} />
            )}

            <Footer path={"home"} />
        </div>
    );
}

export default HomePage;
