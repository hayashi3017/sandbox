"use client";

import CommonButton from "@/components/button/CommonButton";
import { useRouter } from "next/navigation";
import { useState } from "react";

export default function ButtonPage() {
  const router = useRouter();
  const [disabled, setDisabled] = useState(false);

  async function getItems() {
    const response = await fetch("/items");
    const data = await response.json();
    console.log(data);
    return data;
  }

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
