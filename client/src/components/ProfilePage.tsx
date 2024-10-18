import useUserStore from "../store/hooks/useUserStore";
import Background from "./Background";
export default function ProfilePage() {
  const user = useUserStore();

  return (
    <>
      <Background>
        <h1 className="text-6xl text-offwhite">WELCOME {user.username}</h1>
      </Background>
    </>
  );
}
