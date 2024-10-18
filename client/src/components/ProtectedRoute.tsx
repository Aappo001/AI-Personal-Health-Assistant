import useUserStore from "../store/hooks/useUserStore";
import { Navigate, useLocation } from "react-router-dom";
export default function ProtectedRoute({
  children,
}: {
  children: React.ReactNode;
}) {
  const user = useUserStore();
  let location = useLocation();

  if (user.id === -1) {
    console.log("USER ID IS -1, NAVIGATE");
    return <Navigate to={"/login"} state={{ from: location }} replace />;
  }
  console.log("AUTHENTICATED, RETURN CHILDREN");

  return children;
}
