import classes from "./CommonForm.module.css";

type CommonFormProps = {
  onSubmit: () => void;
};

export function CommonForm(props: CommonFormProps) {
  return (
    <form onSubmit={props.onSubmit}>
      <button type="submit" className={classes.button}>Submit</button>
    </form>
  );
}
