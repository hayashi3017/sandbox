import { useState } from "react";
import classes from "./ButtonPage.module.css";
import { CommonButton } from "../../components/button/CommonButton";
import { CommonForm } from "../../components/form/CommonForm";

function ButtonPage() {
  const [count, setCount] = useState(0);
  function increment_count() {
    setCount((count) => count + 1);
  }

  return (
    <>
      <h1>Button Examples</h1>
      <div className={classes.card}>
        <CommonButton onClick={increment_count} disabledOnClick={false}>
          <>count is {count}</>
        </CommonButton>
        <CommonForm onSubmit={increment_count}></CommonForm>
        <p>Explore some pages!</p>
      </div>
      <link href="/button" />
    </>
  );
}

export default ButtonPage;
