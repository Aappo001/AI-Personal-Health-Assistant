import { useRef, useState } from "react";
import { Attachment } from "../../types";

export default function useFileAttachment() {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [attachment, setAttachment] = useState<Attachment>({ file_name: "", file_data: "" });

  const handleFileUploadClick = () => {
    if (!fileInputRef.current) {
      throw new Error("Input ref is null, idk how this happened");
    }
    fileInputRef.current.click();
  };

  const resetFile = () => {
    if (!fileInputRef.current) {
      throw new Error("Input ref is null, idk how this happened");
    }
    fileInputRef.current.value = "";
    setAttachment({ file_name: "", file_data: "" });
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    console.log(JSON.stringify(e.target.files?.[0]));

    if (!file) return;
    const reader = new FileReader();
    reader.onloadend = () => {
      console.log(`Result: ${reader.result}`);
      setAttachment({ file_name: file.name, file_data: reader.result as string });
    };

    reader.readAsDataURL(file);
  };

  const hiddenFileInput = () => {
    return (
      <input type="file" className=" hidden" ref={fileInputRef} onChange={handleFileChange} />
    );
  };

  return {
    attachment,
    hiddenFileInput,
    resetFile,
    handleFileUploadClick,
  };
}
