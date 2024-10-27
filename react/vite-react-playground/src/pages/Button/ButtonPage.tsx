import { useState } from "react";
import classes from "./ButtonPage.module.css";
import { CommonButton } from "../../components/button/CommonButton";
import { CommonForm } from "../../components/form/CommonForm";
import { useNavigate } from "react-router-dom";

function ButtonPage() {
  const navigate = useNavigate();
  const [count, setCount] = useState(0);
  const [disabledClick, setDisabledClick] = useState(false);
  function increment_count() {
    setDisabledClick(true);
    setCount((count) => count + 1);
    setDisabledClick(false);
  }

  async function onClickButton() {
    setDisabledClick(true);
    handle_request();
    setDisabledClick(false);
  }

  async function handle_request() {
    await some_request();
    goToScriptPage();
  }

  async function some_request() {
    const response = await fetch("https://example.com/user");
    const json = await response.json();
    console.log(json);
  }

  async function goToScriptPage() {
    navigate("/script_sync_example");
    // setTimeout(() => {
    //   navigate("/script_sync_example");
    // }, 3000);
  }

  return (
    <>
      <h1>Button Examples</h1>
      <div className={classes.card}>
        <CommonButton onClick={increment_count} disabledOnClick={disabledClick}>
          <>count is {count}</>
        </CommonButton>
        <CommonButton onClick={onClickButton} disabledOnClick={disabledClick}>
          <>start some request</>
        </CommonButton>
        <CommonForm onSubmit={increment_count}></CommonForm>
        <p>Explore some pages!</p>
      </div>
      <link href="/button" />
    </>
  );
}

export default ButtonPage;
