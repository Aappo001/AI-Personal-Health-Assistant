import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend } from "recharts";

const Chart = () => {
  const testData = [
    { date: "2024-11-01", value: 120 },
    { date: "2024-11-02", value: 122 },
    { date: "2024-11-03", value: 119 },
    { date: "2024-11-04", value: 132 },
    { date: "2024-11-05", value: 140 },
  ];

  return (
    <div style={{ textAlign: "center", padding: "20px" }}>
      <h2>Health Data Trends (Test Values)</h2>
      <LineChart
        width={800}
        height={400}
        data={testData}
        margin={{ top: 5, right: 20, bottom: 5, left: 0 }}
      >
        <CartesianGrid stroke="#f5f5f5" />
        <XAxis dataKey="date" />
        <YAxis />
        <Tooltip />
        <Legend />
        <Line type="monotone" dataKey="value" stroke="#8884d8" />
      </LineChart>
    </div>
  );
};

export default Chart;
