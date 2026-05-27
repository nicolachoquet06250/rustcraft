---
name: plan
description: plan de développement avec la stack technique et la roadmap complète à respecter
---

Voici une roadmap réaliste pour un **clone Minecraft-like en Rust**, découpée pour obtenir vite un prototype jouable.

## Stack recommandée

Je partirais sur :

* **Bevy** : moteur Rust ECS, bon pour prototyper un jeu 3D. ([bevy.org][1])
* **wgpu** indirectement via Bevy, ou directement si tu veux ton propre moteur graphique. wgpu est portable Vulkan/Metal/D3D12/WebGPU. ([GitHub][2])
* **Rapier3D** seulement si tu veux une vraie physique. Pour un Minecraft-like, une collision voxel custom suffira au début. Rapier reste solide pour de la physique temps réel. ([rapier.rs][3])
* **openxr** : bindings Rust officieux mais standards pour parler à l’API OpenXR.
* **bevy_openxr** : plugin Bevy/OpenXR si tu veux éviter d’écrire toute l’intégration toi-même.
  oxr / backend OpenXR compatible wgpu : à prévoir si tu fais une intégration plus bas niveau.
  abstraction d’input obligatoire : clavier/souris et contrôleurs VR doivent produire les mêmes PlayerAction.
  abstraction caméra obligatoire : caméra FPS desktop d’un côté, XrRig de l’autre.
  UI 3D VR : parce qu’une UI 2D classique ne suffit pas en casque.

## Architecture cible

```txt
voxel-game/
├─ crates/
│  ├─ core/              # types communs : BlockId, ChunkPos, Direction
│  ├─ world/             # chunks, accès blocs, streaming
│  ├─ terrain/           # génération procédurale
│  ├─ meshing/           # visible faces, greedy meshing
│  ├─ rendering/         # atlas, matériaux, chunk renderer
│  ├─ player/            # état joueur, physique voxel
│  ├─ input/             # actions abstraites desktop/VR
│  ├─ camera/            # caméra FPS + XR rig
│  ├─ interaction/       # raycast blocs desktop/VR
│  ├─ inventory/         # hotbar, items, crafting
│  ├─ save/              # stockage monde
│  ├─ xr/                # session OpenXR, tracking, actions VR
│  ├─ ui/                # UI desktop + UI 3D VR
│  ├─ net/               # futur multijoueur
│  └─ game/              # binaire final Bevy
└─ assets/
   ├─ textures/
   ├─ models/
   └─ shaders/
```

## Roadmap

### Phase 0 — Prototype technique

Objectif : afficher un monde voxel minimal.

À faire :

* fenêtre 3D avec caméra FPS ;
* grille de blocs statiques ;
* déplacement clavier/souris ;
* génération d’un chunk `16x16x16` ou `16x256x16` ;
* rendu naïf : un cube par bloc.

Résultat attendu : tu peux te déplacer autour d’un tas de cubes.

---

### Phase 1 — Chunks et meshing

Objectif : arrêter de rendre un cube complet par bloc.

À faire :

* structure `Chunk`;
* coordonnées monde/chunk/locales ;
* génération de mesh par faces visibles uniquement ;
* suppression des faces internes ;
* chargement/déchargement autour du joueur ;
* cache des chunks modifiés.

Résultat attendu : monde voxel beaucoup plus performant.

---

### Phase 2 — Terrain procédural

Objectif : avoir un monde naturel.

À faire :

* heightmap avec noise ;
* blocs de base : air, grass, dirt, stone, sand, water ;
* biomes simples ;
* arbres basiques ;
* minerais ;
* grottes plus tard.

Résultat attendu : monde explorable type Minecraft alpha.

---

### Phase 3 — Interaction joueur

Objectif : rendre le jeu jouable.

À faire :

* raycast depuis la caméra ;
* casser un bloc ;
* poser un bloc ;
* sélection de bloc ;
* hotbar ;
* collisions joueur contre voxels ;
* gravité et saut.

Résultat attendu : boucle Minecraft minimale : explorer, casser, poser.

---

### Phase 4 — Sauvegarde monde

Objectif : persister les modifications.

À faire :

* format de sauvegarde par chunks ;
* sérialisation avec `serde`;
* compression `zstd` ou `lz4`;
* sauvegarde uniquement des chunks modifiés ;
* chargement prioritaire autour du joueur.

Résultat attendu : les blocs cassés/posés restent après redémarrage.

---

### Phase 5 — Rendu propre

Objectif : améliorer l’aspect visuel.

À faire :

* texture atlas ;
* UV par face ;
* greedy meshing ;
* lumière ambiante ;
* fog/distance ;
* eau semi-transparente ;
* ciel simple ;
* cycle jour/nuit.

Résultat attendu : vrai rendu Minecraft-like.

---

### Phase 6 — Gameplay survival minimal

Objectif : transformer le prototype en jeu.

À faire :

* inventaire ;
* crafting simple ;
* outils ;
* types de blocs avec dureté ;
* drops ;
* vie/faim optionnel ;
* mobs très simples.

Résultat attendu : boucle survival basique.

---

### Phase 7 — Multijoueur

Objectif : serveur dédié authoritative.

À faire :

* serveur Rust séparé ;
* protocole UDP ou QUIC ;
* synchronisation position joueur ;
* synchronisation chunks ;
* modification de blocs ;
* anti-cheat minimal côté serveur ;
* snapshots/deltas.

Résultat attendu : plusieurs joueurs dans le même monde.

---

## Milestones conseillés

| Version | Objectif                   |
|---------|----------------------------|
| `v0.1`  | caméra FPS + chunk visible |
| `v0.2`  | terrain procédural         |
| `v0.3`  | casser/poser blocs         |
| `v0.4`  | collisions + gravité       |
| `v0.5`  | sauvegarde monde           |
| `v0.6`  | textures + greedy meshing  |
| `v0.7`  | inventaire + hotbar        |
| `v0.8`  | crafting                   |
| `v0.9`  | polish solo                |
| `v1.0`  | survival solo stable       |
| `v2.0`  | multijoueur                |

## Ordre de développement recommandé

Ne commence pas par le multijoueur ni par un moteur custom.

Le meilleur ordre :

1. **Bevy + monde voxel simple**
2. **chunks**
3. **meshing optimisé**
4. **raycast blocs**
5. **collisions custom**
6. **sauvegarde**
7. **terrain procédural**
8. **gameplay**
9. **multi**

Le cœur dur du projet, ce n’est pas Rust : c’est **chunking + meshing + streaming + sauvegarde + collisions voxel**.

[1]: https://bevy.org/?utm_source=chatgpt.com "Bevy Engine"
[2]: https://github.com/gfx-rs/wgpu?utm_source=chatgpt.com "gfx-rs/wgpu: A cross-platform, safe, pure-Rust graphics API."
[3]: https://rapier.rs/?utm_source=chatgpt.com "Rapier physics engine | Rapier"
