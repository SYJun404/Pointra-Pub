import type { ToastContentValue } from "@heroui/react";
import {
    Toast,
    ToastContent,
    ToastDescription,
    ToastIndicator,
    ToastTitle,
} from "@heroui/react";

const variantStyles: Record<string, { indicator: string; title: string }> = {
    default: {
        indicator: "text-foreground",
        title: "text-foreground",
    },
    accent: {
        indicator: "text-primary",
        title: "text-primary",
    },
    success: {
        indicator: "text-green-500",
        title: "text-green-500",
    },
    warning: {
        indicator: "text-yellow-500",
        title: "text-yellow-500",
    },
    danger: {
        indicator: "text-red-500",
        title: "text-red-500",
    },
};

function CustomToast({
    placement = "bottom",
}: {
    placement?: "top" | "bottom";
}) {
    return (
        <Toast.Provider placement={placement} className="bottom-13">
            {({ toast: toastItem }) => {
                const content = toastItem.content as ToastContentValue;
                const variant = content.variant ?? "default";
                const styles = variantStyles[variant] ?? variantStyles.default;

                return (
                    <Toast
                        className="rounded-xl border py-2.5 border-border w-fit mx-auto"
                        toast={toastItem}
                        variant={content.variant}
                    >
                        <ToastContent className="flex justify-center items-center">
                            <div className="flex items-center">
                                <ToastIndicator
                                    className={`${styles.indicator} pr-1.5`}
                                    variant={content.variant}
                                />
                                <div className="flex flex-col">
                                    {content.title ? (
                                        <ToastTitle className={styles.title}>
                                            {content.title}
                                        </ToastTitle>
                                    ) : null}
                                    {content.description ? (
                                        <ToastDescription>
                                            {content.description}
                                        </ToastDescription>
                                    ) : null}
                                </div>
                            </div>
                        </ToastContent>
                    </Toast>
                );
            }}
        </Toast.Provider>
    );
}

export default CustomToast;
