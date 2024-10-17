import { Link } from "react-router-dom";
import "./App.css";
import Background from "./components/Background";
import useImplicitLogin from "./store/hooks/useImplicitLogin";
import useUserStore from "./store/hooks/useUserStore";

function App() {
  useImplicitLogin();
  const user = useUserStore();
  return (
    <>
      <Background color="black" className="pt-16">
        <div className="w-full flex flex-col justify-center items-center gap-4">
          <h1 className=" text-6xl text-offwhite">
            AI Personal Health Assistant
          </h1>
          <h1 className=" text-offwhite text-3xl">
            Implicit User: {user.username} {user.firstName}
          </h1>
          <p className=" text-surface75 text-xl">
            An AI assistant that monitors your health and provides suggestions
            for improvement.
          </p>
        </div>
        <div className="w-full flex  justify-center items-center mt-16 gap-8">
          <Link
            to="/register"
            className="px-3 py-5 border-2 font-bold w-64 text-center text-2xl transition-colors duration-150 hover:bg-lilac hover:text-main-black border-lilac text-lilac m-2 rounded-full leading-relaxed"
          >
            Register
          </Link>
          <Link
            to="/login"
            className="px-3 py-5 border-2 font-bold w-64 text-center text-2xl transition-colors duration-150 hover:bg-lilac hover:text-main-black border-lilac text-lilac m-2 rounded-full leading-relaxed"
          >
            Login
          </Link>
        </div>
        <div className="w-full flex justify-center mt-5">
          <Link
            to="/chat"
            className="px-3 py-5 border-2 font-bold w-64 text-center text-2xl transition-colors duration-150 hover:bg-main-green hover:text-main-black border-main-green text-main-green m-2 rounded-full leading-relaxed"
          >
            Chat
          </Link>
        </div>
      </Background>
    </>
  );
}

export default App;
