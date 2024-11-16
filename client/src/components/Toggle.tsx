import { SetStateAction, useState } from "react";

interface Props {
  action: React.Dispatch<SetStateAction<boolean>>;
}
export default function Toggle({ action }: Props) {
  const [enabled, setEnabled] = useState(false);

  const handleChange = () => {
    setEnabled((prev) => !prev);
    action((prev) => !prev);
  };
  return (
    <>
      <label className="inline-flex items-center cursor-pointer">
        <input
          type="checkbox"
          checked={enabled}
          onChange={handleChange}
          className="sr-only peer"
        />
        <div className="relative w-14 h-7 bg-gray-200 peer-focus:outline-none  rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[4px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-6 after:w-6 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
      </label>
    </>
  );
}
