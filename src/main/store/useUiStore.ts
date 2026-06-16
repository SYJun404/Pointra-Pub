import { create } from "zustand";

interface UiState {
    isPinned: boolean;
    updateUrl: string;
    setIsPinned: (isPinned: boolean) => void;
    setUpdateUrl: (updateUrl: string) => void;
}

const useUiStore = create<UiState>()((set) => ({
    isPinned: false,
    updateUrl: "",
    setIsPinned: (isPinned: boolean) => set({ isPinned }),
    setUpdateUrl: (updateUrl: string) => set({ updateUrl }),
}));
export default useUiStore;
