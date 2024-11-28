interface Props {
  fileName: string;
  handleFileClear: () => void;
}
export default function FileAttachment({ fileName, handleFileClear }: Props) {
  return (
    <>
      <div className="border-2 border-lilac relative  rounded-md p-8 flex flex-col items-end justify-between">
        <img
          src="/trash-red.svg"
          className="absolute top-0 right-0 cursor-pointer"
          alt="Cancel"
          onClick={handleFileClear}
        />
        <p className="text-2xl text-lilac mx-auto">{fileName}</p>
      </div>
    </>
  );
}
