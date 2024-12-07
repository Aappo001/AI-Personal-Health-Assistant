import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import axios from "axios";
import Background from "./Background";
import { BASE_URL, getJwt } from "../utils/utils";
import { HealthForm } from "../types";

interface Props {
  name: string;
  units: string;
  yAxisLabel: string;
  dataKey: string;
  callback: (form: HealthForm) => number | undefined;
}

export default function HealthGraph({ name, units, yAxisLabel, dataKey, callback }: Props) {
  const [data, setData] = useState<any []>([]);

  const [responseMessage, setResponseMessage] = useState("");
  const [error, setError] = useState(false);

  const jwt = getJwt();
  const navigate = useNavigate();

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

        const forms = response.data as HealthForm[];
        forms.sort((a, b) => a.id - b.id);

        // Convert the forms data to the format
        const mappedData = forms.map(callback);

        const mappedForm = forms.map((form, index) => {
          const mappedForm: any = {}; 
          mappedForm.date = new Date(form.createdAt).toLocaleDateString();
          mappedForm[dataKey] = mappedData[index]
          return mappedForm;
        }).filter((form) => form[dataKey]);

        setData(mappedForm);
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
      <div className="flex justify-end p-4">
          <button
            onClick={() => navigate("/")}
            className="bg-blue-500 text-white px-4 py-2 rounded-md shadow-md hover:bg-blue-600"
          >
            Home
          </button>
        </div>
        <h1 className="text-4xl font-bebas text-offwhite mb-8">
          {name}
        </h1>
        <div className="w-full max-w-4xl h-auto">
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis
                dataKey="date"
                label={{ value: "Date", position: "insideBottom", offset: -10 }}
              />
              <YAxis
                label={{
                  value: `${yAxisLabel} (${units})`,
                  angle: -90,
                  position: "insideLeft",
                }}
              />
              <Tooltip />
              <Legend />
              <Line
                type="monotone"
                dataKey={dataKey}
                stroke="#8884d8"
                activeDot={{ r: 8 }}
              />
            </LineChart>
          </ResponsiveContainer>
          {responseMessage && (
            <div
              className={`mt-4 text-lg ${error ? "text-red-500" : "text-green-500"
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
