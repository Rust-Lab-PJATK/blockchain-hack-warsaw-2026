import Chat from "./components/Chat";
import styles from "./page.module.css";

export default function Home() {
  return (
    <div className={styles.layout}>
      <Chat />
      <main className={styles.main}>Dashboard placeholder</main>
    </div>
  );
}
