import { useState } from "react";
import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";
import { CommonButton } from "./components/button/CommonButton";
import { CommonForm } from "./components/form/CommonForm";

function App() {
  const [count, setCount] = useState(0);
  function increment_count() {
    setCount((count) => count + 1);
  }

  return (
    <>
      <div>
        <a href="https://vitejs.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
        <CommonButton onClick={increment_count} disabledOnClick={false}>
          <>count is {count}</>
        </CommonButton>
        <CommonForm onSubmit={increment_count}></CommonForm>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  );
}

export default App;
