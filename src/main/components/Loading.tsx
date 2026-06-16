import { Skeleton } from "@heroui/react";

function Loading() {
    return (
        // 外层容器保持与实际内容完全一致的样式
        <div className="mx-3 flex flex-col p-3 pt-2.5 flex-1 min-h-0 border-borderMainW border rounded-xl animate-pulse">
            <main className="flex flex-col gap-2">
                {/* 第一行：原文文本 + 更多按钮 */}
                <div className="flex justify-between items-center">
                    <Skeleton className="h-7 w-24 rounded-md" />{" "}
                    {/* 模拟 text-xl 的高度 */}
                    <Skeleton className="w-6 h-6 rounded-md" />{" "}
                    {/* 模拟省略号按钮 */}
                </div>

                {/* 第二行：音标 + 喇叭按钮 */}
                <div className="flex justify-between items-center">
                    <Skeleton className="h-6 w-24 rounded-md" />{" "}
                    {/* 模拟音标标签 */}
                    <Skeleton className="w-6 h-6 rounded-md" />{" "}
                    {/* 模拟音量按钮 */}
                </div>

                {/* 分割线 */}
                <div className="border-t border-borderSubW my-1 opacity-50"></div>

                {/* 常用释义部分 */}
                <div className="flex flex-col mt-0.5 space-y-2">
                    <Skeleton className="h-3 w-12 rounded" />{" "}
                    {/* 模拟“常用释义”小字 */}
                    <Skeleton className="h-8 w-24  rounded-md" />{" "}
                    {/* 模拟释义大字 */}
                </div>
            </main>

            {/* 底部词典列表部分 */}
            <div className="border-t border-borderSubW pt-1 mt-2 flex flex-col gap-3">
                {[1, 2].map((i) => (
                    <div key={i} className="shrink-0">
                        {/* 词性标签 */}
                        <Skeleton className="h-4 w-8 rounded mb-2" />
                        {/* 释义标签组 */}
                        <div className="flex gap-1">
                            <Skeleton className="h-5 w-12 rounded" />
                            <Skeleton className="h-5 w-16 rounded" />
                            <Skeleton className="h-5 w-10 rounded" />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}

export default Loading;
