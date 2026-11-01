import type { TranslationSource } from "../types/transResult";

const CACHE_LOADING_MIN_MS = 120;

// 从开始显示 loading 时计时，只补足缓存结果剩余的等待时间。
// 联网请求及失败结果不增加延迟；调用方等待后须再次检查请求是否仍有效。
export async function waitForCachePresentation(
    result: { status: number; source: TranslationSource },
    loadingStartedAt: number,
): Promise<void> {
    if (result.status !== 200 || result.source === "network") return;
    const remaining =
        CACHE_LOADING_MIN_MS - (performance.now() - loadingStartedAt);
    if (remaining > 0) {
        await new Promise<void>((resolve) => setTimeout(resolve, remaining));
    }
}
