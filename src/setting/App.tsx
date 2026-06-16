import "../assets/css/App.css";
import useShortcutManager from "./hooks";
import PermissionMacSection from "./components/PermissionMacSection";
import SettingHeader from "./components/SettingHeader";
import GeneralSection from "./components/GeneralSection";
import ShortcutSection from "./components/ShortcutSection";
import AboutSection from "./components/AboutSection";
import CustomToast from "../main/components/CustomToast";

function SettingPage() {
    const {
        shortcuts,
        generalSettings,
        hasChanges,
        recordingId,
        recordingRef,
        conflictIds,
        toggleGeneral,
        updateGeneralSetting,
        startRecording,
        cancelRecording,
        handleKeyDown,
        resetToDefault,
        handleSave,
        closeApp,
        isMac,
        isFirstSetup,
    } = useShortcutManager();

    return (
        <div className="h-screen rounded-4xl flex flex-col bg-white overflow-hidden select-none">
            <CustomToast placement="top" />

            <SettingHeader closeApp={closeApp} />

            {/* ============ 首次设置提示横幅 ============ */}
            {isFirstSetup && (
                <div className="shrink-0 px-3 pt-3">
                    <div className="rounded-xl bg-blueBgW border border-blueBorderW px-4 py-3">
                        <p className="text-xs text-mainBlueW leading-relaxed">
                            🎉 &nbsp; 欢迎使用 Pointra！请先为下方
                            <strong>「授权许可」</strong>
                            中的两项权限授予访问权限，
                            然后点击底部按钮完成设置，应用将自动重启。
                        </p>
                    </div>
                </div>
            )}

            {/* ============ 滚动内容 ============ */}
            <div className="flex-1 overflow-y-auto no-scrollbar px-3 pt-3 mb-3 space-y-6">
                {isMac && <PermissionMacSection />}

                <GeneralSection
                    settings={generalSettings}
                    onToggle={toggleGeneral}
                    onUpdate={updateGeneralSetting}
                />

                <ShortcutSection
                    shortcuts={shortcuts}
                    recordingId={recordingId}
                    recordingRef={recordingRef}
                    conflictIds={conflictIds}
                    onStartRecording={startRecording}
                    onCancelRecording={cancelRecording}
                    onKeyDown={handleKeyDown}
                    onResetOne={resetToDefault}
                    isMac={isMac}
                />

                <AboutSection />
            </div>

            {/* ============ 底部安全区 ============ */}
            {hasChanges && (
                <div className="shrink-0 p-3 border-t border-borderSubW bg-subBgW">
                    <button
                        onClick={handleSave}
                        className="w-full h-10 rounded-full bg-mainBlueW text-white text-sm font-medium cursor-pointer
                                   hover:brightness-110 active:scale-90 transition-all duration-150 shadow-sm"
                    >
                        保存设置
                    </button>
                </div>
            )}
        </div>
    );
}

export default SettingPage;
