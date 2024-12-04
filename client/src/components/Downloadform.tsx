import { useState } from "react";
import Background from "./Background";
import axios from "axios";
import { BASE_URL, getJwt} from "../utils/utils";
import { jsPDF } from "jspdf";

export default function DownloadForm() {
  const [formId, setFormId] = useState<number | undefined>(undefined);
  const [responseMessage, setResponseMessage] = useState("");
  const [error, setError] = useState(false);

  const jwt = getJwt();

  const handleDownload = async () => {
    if (formId === undefined || isNaN(formId)) {
      setResponseMessage("Please enter a valid numeric form ID.");
      setError(true);
      return;
    }

    try {
      setResponseMessage("Fetching forms data...");
      setError(false);

      // Fetch all forms
      const response = await axios.get(`${BASE_URL}/api/forms`, {
        headers: {
          Authorization: `Bearer ${jwt}`,
        },
      });

      // Find the form with the specified ID
      const forms = response.data;
      const form = forms.find((f: any) => f.id === formId);

      if (!form) {
        setResponseMessage(`No form found with ID ${formId}.`);
        setError(true);
        return;
      }

      const { height, weight, sleep_hours, exercise_duration, food_intake, notes } = form;

      // Generate PDF
      const doc = new jsPDF();
      doc.setFont("helvetica", "bold");
      doc.setFontSize(18);
      doc.text("Health Statistics Report", 20, 20);

      doc.setFont("helvetica", "normal");
      doc.setFontSize(14);
      doc.text(`Form ID: ${formId}`, 20, 40);
      doc.text(`Height: ${height} cm`, 20, 50);
      doc.text(`Weight: ${weight} kg`, 20, 60);
      doc.text(`Sleep Hours: ${sleep_hours}`, 20, 70);
      doc.text(`Exercise Duration: ${exercise_duration} minutes`, 20, 80);
      doc.text(`Food Intake: ${food_intake}`, 20, 90);
      doc.text(`Notes: ${notes}`, 20, 100);

      const fileName = `health_stats_${formId}.pdf`;
      doc.save(fileName);

      setResponseMessage(`PDF downloaded successfully as "${fileName}".`);
      setError(false);
    } catch (error: any) {
      const errorMessage =
        error.response?.data?.message ||
        "An error occurred while fetching the forms. Please try again.";
      setResponseMessage(errorMessage);
      setError(true);
    }
  };

  return (
    <Background color="black">
      <div className="w-full h-full flex flex-col justify-center items-center">
        <div className="border-2 border-offwhite px-6 py-6 flex flex-col items-center gap-6 w-3/12 h-auto rounded-sm">
          <p className="text-offwhite text-4xl font-bebas">Download Form</p>
          <input
            type="number"
            placeholder="Enter Form ID"
            value={formId ?? ""}
            onChange={(e) => setFormId(e.target.value ? parseInt(e.target.value, 10) : undefined)}
            className="w-full px-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
          />
          <button
            onClick={handleDownload}
            className="px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black"
          >
            Download PDF
          </button>
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
