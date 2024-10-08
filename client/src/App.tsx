import { Link } from "react-router-dom";
import "./App.css";

function App() {
  return (
    <>
      <Link
        to="/register"
        className="px-4 py-2 border-[1px] border-black m-2 rounded-sm"
      >
        Register
      </Link>
      <Link
        to="/login"
        className="px-4 py-2 border-[1px] border-black m-2 rounded-sm"
      >
        Login
      </Link>
    </>
  );
}

export default App;
