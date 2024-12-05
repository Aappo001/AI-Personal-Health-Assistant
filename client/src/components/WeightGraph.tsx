import { useEffect, useState } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from "recharts";
import axios from "axios";
import Background from "./Background";
import { BASE_URL, getJwt } from "../utils/utils";

export default function Graph() {
  const [data, setData] = useState([]);
  const [responseMessage, setResponseMessage] = useState("");
  const [error, setError] = useState(false);

  const jwt = getJwt();

  useEffect(() => {
    const fetchData = async () => {
      try {
        setResponseMessage("Fetching graph data...");
        setError(false);

        const response = await axios.get(`${BASE_URL}/api/forms`, {
          headers: {
            Authorization: `Bearer ${jwt}`,
          },
        });

        const forms = response.data
          .map((form: any) => ({
            id: form.id, 
            date: form.modified_at.slice(0, 10),
            weight: form.weight,
          }))
          .sort((a: any, b: any) => a.id - b.id);

        setData(forms);
        setResponseMessage("Data loaded successfully.");
      } catch (error: any) {
        const errorMessage =
          error.response?.data?.message ||
          "An error occurred while fetching the graph data. Please try again.";
        setResponseMessage(errorMessage);
        setError(true);
      }
    };

    fetchData();
  }, [jwt]);

  return (
    <Background color="black">
      <div className="w-full h-full flex flex-col justify-center items-center">
        <div className="w-full max-w-4xl h-auto">
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="date" 
                label={{ value: "Date", position: "insideBottom", offset: -10 }} 
              />
              <YAxis 
                label={{ value: "Weight (kg)", angle: -90, position: "insideLeft" }} 
              />
              <Tooltip />
              <Legend />
              <Line type="monotone" dataKey="weight" stroke="#8884d8" activeDot={{ r: 8 }} />
            </LineChart>
          </ResponsiveContainer>
          {responseMessage && (
            <div
              className={`mt-4 text-lg ${
                error ? "text-red-500" : "text-green-500"
              }`}
            >
              {responseMessage}
            </div>
          )}
        </div>
      </div>
    </Background>
  );
}
