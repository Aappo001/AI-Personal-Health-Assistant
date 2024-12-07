import { useNavigate } from "react-router-dom";
import Background from "./Background";

export default function UserStats() {
  const navigate = useNavigate();

  return (
    <Background color="black">
            <div className="flex justify-end p-4">
          <button
            onClick={() => navigate("/")}
            className="bg-blue-500 text-white px-4 py-2 rounded-md shadow-md hover:bg-blue-600"
          >
            Home
          </button>
        </div>
      <div className="w-full h-full flex flex-col items-center justify-center">
        {/* Title */}
        <h1 className="text-4xl font-bold text-offwhite mb-8 font-bebas">
          User Stats
        </h1>

        {/* Buttons */}
        <div className="flex flex-col gap-4 w-64">
        <button
            onClick={() => navigate("/userprofileform")}
            className="px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black"
          >
            Create User Profile
          </button>
          <button
            onClick={() => navigate("/healthform")}
            className="px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black"
          >
            Create Form
          </button>
          <button
            onClick={() => navigate("/downloadform")}
            className="px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black"
          >
            Download Form
          </button>
          <button
            onClick={() => navigate("/weightgraph")}
            className="px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black"
          >
            Weight Graph
          </button>
          <button
            onClick={() => navigate("/exercisegraph")}
            className="px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black"
          >
            Exercise Graph
          </button>
        </div>
      </div>
    </Background>
  );
}
