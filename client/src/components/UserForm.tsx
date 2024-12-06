import { useState } from "react";
import Background from "./Background";
import axios from "axios";
import { BASE_URL, getJwt } from "../utils/utils";
import { useNavigate } from "react-router-dom";


interface HealthStatsBody {
    height?: number;
    weight?: number;
    sleepHours?: number;
    exerciseDuration?: number;
    foodIntake?: string;
    notes?: string;
}

export default function UserHealthForm() {



    const [healthStats, setHealthStats] = useState<HealthStatsBody>({
        height: undefined,
        weight: undefined,
        sleepHours: undefined,
        exerciseDuration: undefined,
        foodIntake: "",
        notes: "",
    });
    const [responseMessage, setResponseMessage] = useState<string>("");
    const [error, setError] = useState(false);
    const [isSubmitting, setIsSubmitting] = useState(false);


    const jwt = getJwt();
    const navigate = useNavigate();

    const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setIsSubmitting(true);
        setResponseMessage("");
        setError(false);

        // Basic validation
        if (!healthStats.height || !healthStats.weight) {
            setResponseMessage("Height and Weight are required fields.");
            setError(true);
            setIsSubmitting(false);
            return;
        }

        try {
            const response = await axios.post(
                `${BASE_URL}/api/forms/health`,
                healthStats,
                {
                    headers: {
                        Authorization: `Bearer ${jwt}`,
                        "Content-Type": "application/json",
                    },
                }
            );

            setResponseMessage(`Health stats submitted successfully! Entry ID: ${response.data.id}`);
            setError(false);

            // Clear form fields after submission
            setHealthStats({
                height: undefined,
                weight: undefined,
                sleepHours: undefined,
                exerciseDuration: undefined,
                foodIntake: "",
                notes: "",
            });
        } catch (error: any) {
            setResponseMessage(
                error.response ? error.response.data.message : "An error occurred. Please try again later."
            );
            setError(true);
        } finally {
            setIsSubmitting(false);
        }
    };

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
            <div className="w-full h-full flex flex-col justify-center items-center">
                <div className="border-2 border-offwhite px-4 py-4 flex flex-col items-center justify-evenly gap-4 w-3/12 h-auto rounded-sm">
                    <p className={`text-offwhite text-6xl font-bebas`}>Health Stats</p>
                    <form
                        className="flex flex-col justify-center items-center gap-5 w-5/6"
                        onSubmit={handleSubmit}
                    >
                        <input
                            type="number"
                            name="height"
                            value={healthStats.height || ""}
                            onChange={(e) =>
                                setHealthStats({
                                    ...healthStats,
                                    height: parseFloat(e.target.value) || undefined,
                                })
                            }
                            placeholder="Height (cm)"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <input
                            type="number"
                            name="weight"
                            value={healthStats.weight || ""}
                            onChange={(e) =>
                                setHealthStats({
                                    ...healthStats,
                                    weight: parseFloat(e.target.value) || undefined,
                                })
                            }
                            placeholder="Weight (kg)"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <input
                            type="number"
                            name="sleep_hours"
                            value={healthStats.sleepHours || ""}
                            onChange={(e) =>
                                setHealthStats({
                                    ...healthStats,
                                    sleepHours: parseFloat(e.target.value) || undefined,
                                })
                            }
                            placeholder="Sleep Hours"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <input
                            type="number"
                            name="exercise_duration"
                            value={healthStats.exerciseDuration || ""}
                            onChange={(e) =>
                                setHealthStats({
                                    ...healthStats,
                                    exerciseDuration: parseFloat(e.target.value) || undefined,
                                })
                            }
                            placeholder="Exercise Duration (minutes)"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <textarea
                            name="food_intake"
                            value={healthStats.foodIntake || ""}
                            onChange={(e) =>
                                setHealthStats({
                                    ...healthStats,
                                    foodIntake: e.target.value,
                                })
                            }
                            placeholder="Describe Food Intake"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <textarea
                            name="notes"
                            value={healthStats.notes || ""}
                            onChange={(e) =>
                                setHealthStats({
                                    ...healthStats,
                                    notes: e.target.value,
                                })
                            }
                            placeholder="Additional Notes (e.g., how you felt, specific activities)"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <button
                            type="submit"
                            disabled={isSubmitting}
                            className={`px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black ${isSubmitting ? "opacity-50 cursor-not-allowed" : ""
                                }`}
                        >
                            {isSubmitting ? "Submitting..." : "Submit"}
                        </button>
                    </form>
                    {responseMessage && (
                        <div
                            className={`mt-4 text-xl ${error ? "text-red-500" : "text-green-500"}`}
                        >
                            {responseMessage}
                        </div>
                    )}
                </div>
            </div>
        </Background>
    );
}
