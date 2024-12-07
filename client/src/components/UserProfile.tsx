import { useState } from "react";
import Background from "./Background";
import axios from "axios";
import { BASE_URL, getJwt } from "../utils/utils";

interface UserProfileBody {
    first_name?: string;
    last_name?: string;
    phone_number?: string;
    image?: File;
}

export default function UserProfileForm() {
    const [formData, setFormData] = useState<UserProfileBody>({
        first_name: "",
        last_name: "",
        phone_number: "",
    });
    const [selectedFile, setSelectedFile] = useState<File | null>(null);
    const [responseMessage, setResponseMessage] = useState<string>("");
    const [error, setError] = useState(false);
    const [isSubmitting, setIsSubmitting] = useState(false);

    const jwt = getJwt();

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const { name, value } = e.target;
        setFormData((prev) => ({
            ...prev,
            [name]: value,
        }));
    };

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files[0]) {
            setSelectedFile(e.target.files[0]);
        }
    };

    const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        setIsSubmitting(true);
        setResponseMessage("");
        setError(false);

        // Validation
        if (!formData.first_name || !formData.last_name || !formData.phone_number) {
            setResponseMessage("First Name, Last Name, and Phone Number are required.");
            setError(true);
            setIsSubmitting(false);
            return;
        }

        try {
            const submissionData = new FormData();
            submissionData.append("first_name", formData.first_name || "");
            submissionData.append("last_name", formData.last_name || "");
            submissionData.append("phone_number", formData.phone_number || "");

            if (selectedFile) {
                submissionData.append("image", selectedFile);
            }

            const response = await axios.post(`${BASE_URL}/api/forms/user_profile`, 
                submissionData, {
                headers: {
                    Authorization: `Bearer ${jwt}`,
                    "Content-Type": "multipart/form-data",
                },
            });

            setResponseMessage(`Profile submitted successfully! ID: ${response.data.id}`);
            setError(false);
            setFormData({
                first_name: "",
                last_name: "",
                phone_number: "",
            });
            setSelectedFile(null);
        } catch (err: any) {
            setResponseMessage(
                err.response ? err.response.data.message : "An error occurred. Please try again later."
            );
            setError(true);
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <Background color="black">
            <div className="w-full h-full flex flex-col justify-center items-center">
                <div className="border-2 border-offwhite px-4 py-4 flex flex-col items-center justify-evenly gap-4 w-3/12 h-auto rounded-sm">
                    <p className={`text-offwhite text-6xl font-bebas`}>User Profile</p>
                    <form
                        className="flex flex-col justify-center items-center gap-5 w-5/6"
                        onSubmit={handleSubmit}
                    >
                        <input
                            type="string"
                            name="first_name"
                            value={formData.first_name || ""}
                            onChange={handleInputChange}
                            placeholder="First Name"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <input
                            type="string"
                            name="last_name"
                            value={formData.last_name || ""}
                            onChange={handleInputChange}
                            placeholder="Last Name"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <input
                            type="string"
                            name="phone_number"
                            value={formData.phone_number || ""}
                            onChange={handleInputChange}
                            placeholder="Phone Number (No hyphens)"
                            className="w-full mt-4 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
                        />
                        <input
                            type="file"
                            accept="image/*"
                            onChange={handleFileChange}
                            className="w-full mt-4 pr-4 py-2 placeholder:text-surface75 focus:outline-none transition-colors duration-200 bg-main-black text-offwhite"
                        />
                        <button
                            type="submit"
                            disabled={isSubmitting}
                            className={`px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black ${
                                isSubmitting ? "opacity-50 cursor-not-allowed" : ""
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
