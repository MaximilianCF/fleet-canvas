# Lições Aprendidas — Open-RMF com Robô Dubin/VDA5050

> Documento de lições aprendidas na integração do robô Freebotics
> mutley01 (cinemática Dubin, protocolo VDA5050) ao stack Open-RMF.
>
> Estas lições motivaram os validators do §7 do ROADMAP e estão
> codificadas em `site/charger_lint.rs`, `site/anchor.rs` (merge),
> `site/lane_arrows.rs` e na página 7 do Feature Guide do editor.

---

## 1. RMF pressupõe DifferentialDrive

O planner do Open-RMF assume que o robô **rotaciona em torno do próprio
eixo** e pode **reverter**. Dubin/Ackermann/Tugger não fazem nenhum dos
dois. Consequência: sem mitigação, o planner gera trajetórias que o
robô é incapaz de executar (curvas fechadas, manobras de ré).

**Mitigação:** forçar one-way em todas as lanes (validado pelo
`check_bidirectional_lanes_for_non_diff`) e recalcular theta no fleet
adapter com `atan2` a cada waypoint, ignorando o theta do planner.

---

## 2. Theta offset no dashboard do RMF

O dashboard web (`robot-three.tsx`, upstream) subtrai **π** do yaw do
robô ao renderizar. Se o fleet adapter publica o yaw real, o robô
aparece virado 180° no mapa.

**Mitigação:** `get_data()` no RobotClientAPI soma +π ao yaw antes de
publicar. Em `navigate()`, **nunca** use o theta do planner diretamente
— recalcule com `atan2(dy, dx)` entre waypoints consecutivos.

---

## 3. Lanes bidirecionais são proibidas

Cada `ReverseLane::Same` ou `ReverseLane::Different(_)` em um grafo
usado por robô não-reversível é uma falha esperando acontecer. O
planner tentará reverter, o robô não consegue, a tarefa pendura.

**Regra:** toda lane tem `ReverseLane::Disable`. Para corredor de duas
mãos, desenhe **duas lanes paralelas** em sentidos opostos.

**Ferramenta:** o validator `check_bidirectional_lanes_for_non_diff`
flaga todas as lanes bidirecionais no painel de diagnóstico quando há
um `DifferentialDrive { bidirectional: false }` no site.

---

## 4. Cantos retos matam o Dubin

Conectar duas lanes retas em 90° diretamente gera uma trajetória com
raio de curvatura zero — impossível para qualquer robô com raio mínimo
de curva.

**Regra:** todo canto precisa de **2 a 3 waypoints intermediários**
formando um arco. Cada segmento do arco deve respeitar o raio mínimo
do robô. Na prática, ~45° por segmento funciona para a maioria dos
robôs comerciais.

---

## 5. Ordem do desenho importa

Desenhar a nav graph na ordem errada resulta em um emaranhado com lanes
duplicadas, direções invertidas e anchors sobrepostas.

**Procedimento:**
1. Coloque **todos os waypoints primeiro**, sem nenhuma lane.
2. Desenhe as lanes em sequência, seguindo o fluxo esperado do robô
   (origem → destino define a direção).
3. Desmarque `bidirectional` **imediatamente** após cada lane (antes
   de perder o contexto).
4. Feche o loop: a última lane conecta de volta ao primeiro waypoint.

---

## 6. Anchors duplicadas são silenciosas

Ao arrastar um waypoint sobre outro durante a edição, é fácil acabar
com duas anchors no mesmo ponto sem que nenhuma aresta as una. O robô
pára ou se comporta de forma errática porque o planner vê o grafo
desconectado.

**Ferramenta:** `check_for_close_unconnected_anchors` detecta pares de
anchors a < 0.2 m uma da outra sem lane entre elas. No painel de
Diagnostics, use o botão **Merge ↗** para fundir as duas em uma só —
todas as lanes, walls e locations são reapontadas automaticamente.

---

## 7. Nome do charger = string no config.yaml

O fleet adapter do RMF localiza o charger pelo **nome exato** no
`config.yaml`. Nome em branco, nome diferente, ou nomes duplicados em
grafos diferentes causam falha silenciosa de dispatch de carga — a
tarefa fica pendurada, o robô não carrega, e não há erro óbvio no log.

**Ferramenta:** `check_charger_waypoints` flaga:
- Locations com `LocationTag::Charger` e `NameInSite` vazio.
- Chargers com nome duplicado dentro do mesmo nav graph.

---

## 8. Reference coordinates precisam ser medidas

O campo `reference_coordinates` no `config.yaml` do fleet adapter
define a transformação de similaridade 2D entre o frame do RMF (nav
graph) e o frame do ROS 2 do robô (map/odom). Estimar no olhômetro
quase nunca funciona — o erro composto resulta em waypoints com 30 cm
de offset na execução.

**Procedimento correto:**
1. Leve o robô a 2–3 pontos conhecidos no mapa físico.
2. Anote a posição (x, y) reportada pelo fleet adapter em ROS 2.
3. Anote a posição (x, y) do waypoint correspondente na nav graph.
4. Calcule a transformação de similaridade (scale, rotation, translation)
   via mínimos quadrados.
5. Cole os coeficientes em `reference_coordinates`.

**TODO futuro (§7 pending):** ferramenta no editor que aceita pares
(ponto clicado no mapa, pose ROS 2 digitada) e calcula a
transformação automaticamente, exportando YAML pronto.

---

## 9. VDA5050 é um envelope, não uma semântica

VDA5050 descreve **como** enviar ordens a um robô (tópicos MQTT,
esquema JSON de `order`/`state`) mas **não** descreve os limites
cinemáticos dele. Um robô VDA5050 Dubin e um VDA5050 diferencial falam
o mesmo protocolo mas têm capacidades diametralmente opostas.

**Consequência:** não assuma nada sobre a capacidade do robô a partir
de "é VDA5050". Sempre verifique a ficha técnica e modele o
`DifferentialDrive.bidirectional` de acordo.

---

## 10. Valide após cada mudança

A fleet completa (editor → SDF export → Gazebo → fleet adapter → dashboard)
tem ~8 pontos de falha. Descobrir qual quebrou depois de 30 minutos
iterando é frustrante.

**Regra:** rode `Tools > Diagnostic Tool > Validate` **após cada mudança
na nav graph**. Os validators do §1 e §7 cobrem as falhas mais comuns
antes que cheguem ao fleet adapter:

- Conectividade do grafo (`check_for_disconnected_nav_graph_components`)
- Clearance de walls (`check_lane_clearance_to_walls`)
- Reachability de doors/lifts (`check_door_lift_reachability`)
- Anchors duplicadas (`check_for_close_unconnected_anchors`)
- Nome de charger (`check_charger_waypoints`)
- Bidirectional em robô não-reversível (`check_bidirectional_lanes_for_non_diff`)

Se todos passam, o erro está no fleet adapter ou no config — o site
editor saiu do caminho.
