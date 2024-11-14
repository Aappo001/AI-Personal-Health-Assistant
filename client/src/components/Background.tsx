interface Props {
  color?: "black" | "offwhite";
  children: React.ReactNode;
  className?: string;
}
export default function Background({ color = "black", children, className }: Props) {
  return (
    <div
      className={` w-screen h-screen overflow-auto ${
        color === "black" ? "bg-main-black" : "bg-offwhite"
      } ${className} `}
    >
      {children}
    </div>
  );
}
