type CommonButtonProps = {
  text: string;
  onClick: () => void;
  disabled: boolean;
};

export default function CommonButton(props: CommonButtonProps) {
  return (
    <button onClick={props.onClick} disabled={props.disabled}>
      {props.text}
    </button>
  );
}
