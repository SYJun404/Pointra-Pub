import { useOnWindowChange } from "../utils/useCustom";
import { listen } from "@tauri-apps/api/event";
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

    useEffect(() => {
        if (location.state?.word) {
            fetchTranslate(location.state.word);
        }

        const setupListener = async () => {
            const unlisten = await listen<string>(
                "from-cursor",
                async (event) => {
                    await fetchTranslate(event.payload);
                },
            );

            // 返回清理函数
            return unlisten;
        };

        const listenerPromise = setupListener();

        return () => {
            listenerPromise.then((unlisten) => unlisten());
        };
    }, []);

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
