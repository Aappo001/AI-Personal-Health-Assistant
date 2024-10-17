// Symptom class remains the same

import java.util.HashMap;
import java.util.Map;

class Symptom {
    private String name;
    private Map<String, String> severityToSolution;

    public Symptom(String name) {
        this.name = name;
        this.severityToSolution = new HashMap<>();
    }

    public void addSolution(String severity, String solution) {
        severityToSolution.put(severity, solution);
    }

    public String getSolution(String severity) {
        return severityToSolution.get(severity);
    }

    public String getName() {
        return name;
    }
}