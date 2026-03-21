import { create } from 'zustand';

interface AppStore {
  isDarkMode: boolean;
  toggleDarkMode: () => void;
}

export const useAppStore = create<AppStore>((set) => ({
  isDarkMode: false,
  toggleDarkMode: () => set((state) => ({ isDarkMode: !state.isDarkMode })),
}));

