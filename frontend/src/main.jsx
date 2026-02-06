import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import App from "./App";
import NewDoc from "./NewDoc";
import "./styles/index.css";

createRoot(document.getElementById("root")).render(
  <BrowserRouter>
    <Routes>
      <Route path="/d/:docId" element={<App />} />
      <Route path="*" element={<NewDoc />} />
    </Routes>
  </BrowserRouter>,
);
