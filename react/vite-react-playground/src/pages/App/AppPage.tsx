import classes from "./AppPage.module.css";

function AppPage() {
  return (
    <>
      <h1>Home</h1>
      <h2>Examples</h2>
      <div className={classes.card}>
        <ul>
          <li>
            <a href="/">Home</a>
          </li>
          <li>
            <a href="/button">Button</a>
          </li>
          <li>
            <a href="/script_sync_example">Script</a>
          </li>
        </ul>
      </div>
    </>
  );
}

export default AppPage;
