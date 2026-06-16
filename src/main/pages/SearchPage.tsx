import { Magnifier, Xmark, Hashtag } from "@gravity-ui/icons";
import { useState, useRef, useEffect } from "react";
import Header from "../components/Header";
import Footer from "../components/Footer";
import { TransResultZHTypes } from "../types/transResult";
import { invoke } from "@tauri-apps/api/core";
import SearchContent from "../components/SearchContent";
import { useOnWindowChange } from "../utils/useCustom";
import { useNavigate, useLocation } from "react-router-dom";
import { toast } from "@heroui/react";
import CustomToast from "../components/CustomToast";
import { Spinner } from "@heroui/react";

function SearchPage({ isMac }: { isMac: boolean }) {
    const navigate = useNavigate();
    const location = useLocation();
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const [input, setinput] = useState<string>("");
    const [results, setResults] = useState<TransResultZHTypes | null>(null);
    const [loading, setLoading] = useState(false);

    // 窗口显示时执行的回调
    useOnWindowChange(() => {
        navigate("/");
    });

    const handleSubmit = async (e: any) => {
        e.preventDefault();
        inputRef.current?.blur();
        if (input.trim() === "") return;
        setLoading(true);

        try {
            const res = await invoke<TransResultZHTypes>("fetch_trans_res", {
                word: input,
            });
            if (res.status === 200) {
                setResults(res);
            } else {
                setResults(null);
            }
        } catch (err) {
            console.error(err);
            setResults(null);
        } finally {
            setLoading(false);
        }
    };

    const changeInput = (e: any) => {
        setinput(e.target.value);
        if (e.target.value === "") {
            setResults(null);
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleSubmit(e);
        }
    };

    const handleCopy = async () => {
        try {
            if (results === null) return;
            await navigator.clipboard.writeText(results.data.translate.dit);
            toast.success("复制成功!", {
                timeout: 1500,
            });
        } catch (err) {
            toast.danger("复制失败!", {
                timeout: 1500,
            });
        }
    };

    useEffect(() => {
        const textarea = inputRef.current;
        if (textarea) {
            textarea.style.height = "auto";
            textarea.style.height = textarea.scrollHeight + "px";
        }
    }, [input]);

    useEffect(() => {
        if (location.state) {
            const res: TransResultZHTypes = location.state.data;
            setinput(res.data.translate.text);
            setResults(res);
        } else {
            inputRef.current?.focus();
        }
    }, []);

    return (
        <div className="pt-3 flex flex-col gap-3 h-screen overflow-hidden">
            <CustomToast />
            <Header />
            {/* 顶部栏 */}
            <div className="mx-3 flex items-center gap-3">
                <div className="flex-1 relative">
                    <form onSubmit={handleSubmit}>
                        <textarea
                            ref={inputRef}
                            value={input}
                            onChange={changeInput}
                            onKeyDown={handleKeyDown}
                            rows={1}
                            placeholder="Type Something..."
                            className="w-full min-h-9 px-8 py-1.75 text-sm rounded-[10px] bg-white border border-borderMainW
                                   outline-none focus:border-mainBlueW transition-colors text-mainTitleW
                                   resize-none overflow-hidden max-h-23 overflow-y-auto no-scrollbar cursor-default"
                        />
                    </form>
                    <Magnifier
                        color="#bbbbbb"
                        height={14}
                        width={14}
                        className="absolute left-3 top-3"
                    />
                    <Xmark
                        onClick={() => {
                            setinput("");
                            setResults(null);
                        }}
                        color="#bbbbbb"
                        height={14}
                        width={14}
                        className="absolute right-3 top-3 cursor-pointer"
                    />
                </div>
            </div>

            {/* 搜索结果区域 */}
            <div className="mx-3 flex-1 border border-borderMainW rounded-xl overflow-y-auto no-scrollbar">
                {loading ? (
                    <div className="flex flex-col items-center mt-20 h-full gap-2">
                        <Spinner className="text-mainBlueW" />
                        <p className="text-sm text-tagW">Loading...</p>
                    </div>
                ) : results ? (
                    results.data.wordCard?.secondQuery === undefined ||
                    typeof results.data.wordCard.secondQuery === "string" ? (
                        <div className="p-3">
                            <div className="flex items-center gap-1.5 mb-0.5">
                                <Hashtag
                                    color="#4a90d9"
                                    height={14}
                                    width={14}
                                />

                                <span className="text-xs  font-medium text-tagW tracking-wider">
                                    翻译结果
                                </span>
                            </div>
                            <p
                                onClick={handleCopy}
                                className="text-sm leading-relaxed text-mainTitleW cursor-pointer
                                       transition-all duration-200 hover:text-mainBlueW active:scale-[0.98]
                                       select-none"
                            >
                                {results.data.translate.dit}
                            </p>
                        </div>
                    ) : (
                        <SearchContent
                            results={results.data.wordCard.secondQuery}
                        />
                    )
                ) : (
                    <div>
                        <p className="text-sm text-tagW text-center mt-20">
                            输入文字，按 Enter 翻译
                        </p>
                        {!isMac && (
                            <p className="text-[11px] text-tagW text-center ">
                                请勿将光标移动到输入框内
                            </p>
                        )}
                    </div>
                )}
            </div>
            <Footer path={"search"} />
        </div>
    );
}

export default SearchPage;
