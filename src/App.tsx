import { useEffect } from "react";
import { Capsule } from "./components/Capsule/Capsule";
import { useAppStore } from "./stores/appStore";
import "./App.css";

export default function App() {
  const theme = useAppStore((state) => state.theme);
  const loadTheme = useAppStore((state) => state.loadTheme);
  useEffect(() => { void loadTheme(); }, [loadTheme]);
  return <div className="capsule-app" data-theme={theme}><Capsule /></div>;
}
