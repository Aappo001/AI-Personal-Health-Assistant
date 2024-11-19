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
          <h1 className="text-6xl text-offwhite">
            AI Personal Health Assistant
          </h1>
          {user.id !== -1 && (
            <h1 className="text-offwhite text-3xl underline">
              Welcome back, {user.username}
            </h1>
          )}
          <p className="text-surface75 text-xl">
            An AI assistant that monitors your health and provides suggestions
            for improvement.
          </p>
        </div>
        <div className="w-full flex justify-center items-center mt-16 gap-8">
          {user.id !== -1 && (
            <Link
              to={`/profile/${user.username}`}
              className="px-3 py-5 border-2 font-bold w-96 text-center text-2xl transition-colors duration-150 hover:bg-lilac hover:text-main-black border-lilac text-lilac m-2 rounded-full leading-relaxed"
            >
              Edit Profile
            </Link>
          )}
          {user.id === -1 && (
            <>
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
            </>
          )}
        </div>
        <div className="w-full flex justify-center mt-5">
          <Link
            to="/chat"
            className="px-3 py-5 border-2 font-bold w-64 text-center text-2xl transition-colors duration-150 hover:bg-main-green hover:text-main-black border-main-green text-main-green m-2 rounded-full leading-relaxed"
          >
            Chat
          </Link>
        </div>
        <div className="w-full flex justify-center mt-5">
          <Link
            to="/ws"
            className="px-3 py-5 border-2 font-bold w-64 text-center text-2xl transition-colors duration-150 hover:bg-main-blue hover:text-main-black border-main-blue text-main-blue m-2 rounded-full leading-relaxed"
          >
            Websocket Testing
          </Link>
        </div>
        <div className="w-full flex justify-center mt-5">
          <Link
            to="/health-form"
            className="px-3 py-5 border-2 font-bold w-64 text-center text-2xl transition-colors duration-150 hover:bg-red-500 hover:text-main-black border-red-500 text-red-500 m-2 rounded-full leading-relaxed"
          >
            User Health Form
          </Link>
        </div>
      </Background>
    </>
  );
}

export default App;
