import CommonButton from "@/components/button/CommonButton";
import { useRouter } from "next/router";
import { useState } from "react";

export default function ButtonPage() {
  const router = useRouter();
  const [disabled, setDisabled] = useState(false);

  async function getItems() {
    const response = await fetch("/api/hello");
    const data = await response.json();
    console.log(data);
    return data;
  }

  // Next.js v12の場合、この書き方ではrouter.push()を完全にawaitできなかったが、現在はできるように修正されている模様
  async function redirectTopPage() {
    await router.push("/");
  }

  async function innerOnClick() {
    await getItems();
    await redirectTopPage();
  }

  async function onClickButton() {
    setDisabled(true);
    await innerOnClick();
    setDisabled(false);
  }

  return (
    <>
      <h1>Example: Button double click</h1>
      <CommonButton text="submit" onClick={onClickButton} disabled={disabled} />
    </>
  );
}
