import classes from "./CommonButton.module.css";
import { ReactElement } from "react";

type CommonButtonProps = {
  onClick: () => void;
  children: ReactElement;
  disabledOnClick: boolean;
};

export function CommonButton(props: CommonButtonProps) {
  return (
    <button
      onClick={props.onClick}
      disabled={props.disabledOnClick}
      className={classes.default}
    >
      {props.children}
    </button>
  );
}
