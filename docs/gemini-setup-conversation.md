## Journal de Conversation Gemini

*   **Conversation de Setup Initiale :** https://gemini.google.com/share/271ee2eba1f9
*   **Historique complet de la conversation :** Fourni localement.

*Note : En tant qu'IA, je ne peux pas accéder aux liens externes. Pour le contexte, veuillez coller les extraits de code et de conversation pertinents directement dans le chat.*

**Processus de Documentation :**
*   Nous utiliserons des **Architecture Decision Records (ADR)** dans le dossier `/docs/adr` pour documenter les décisions techniques importantes.
*   Chaque module Rust aura des commentaires `rustdoc` pour l'API publique.

**Changement Structurel :**
*   Le projet a été converti en **Cargo Workspace** pour mieux gérer les futurs composants (`uto-core`, `uto-vision`, etc.).
*   Les commandes `cargo` doivent maintenant être exécutées depuis le répertoire racine `UTO`.

**Prochaines Étapes :** Continuer l'implémentation de la logique de découverte dans `uto-env`.
