# Lições Aprendidas — Open-RMF com Robô Dubin/VDA5050

Documento de lições aprendidas na integração do robô Freebotics 
mutley01 (cinemática Dubin, eixo traseiro, sem ré, sem giro no 
eixo) com Open-RMF via fleet adapter VDA5050.

**Data:** 2026-04-15  
**Robô:** Freebotics mutley01 (agvKinematic: THREEWHEEL, agvClass: TUGGER)  
**Protocolo:** VDA5050 v2.0 sobre MQTT

# Lições Aprendidas — Open-RMF com Robô Dubin/VDA5050

Documento de lições aprendidas na integração do robô Freebotics mutley01 (cinemática Dubin, eixo traseiro, sem ré, sem giro no eixo) com Open-RMF via fleet adapter VDA5050.

**Data:** 2026-04-15  
**Robô:** Freebotics mutley01 (agvKinematic: THREEWHEEL, agvClass: TUGGER)  
**Protocolo:** VDA5050 v2.0 sobre MQTT

---

## 1. Charger deve usar o nome exato do waypoint no nav graph

O `config.yaml` tinha `charger: "Ponto de carga"` mas o waypoint no nav graph se chamava `"F"`. O RMF recusa adicionar o robô à frota se não encontra o waypoint do charger pelo nome exato. O erro é fatal — o robô simplesmente não aparece na lista e o `update_handle` nunca é criado, gerando cascata de erros `'NoneType' object has no attribute 'more'` no update_loop.

**Erro no log:**
```
Cannot find a waypoint named [Ponto de carga] in the navigation graph of fleet [freebotics] 
needed for the charging point of robot [mutley01]. We will not add the robot to the fleet.
```

**Regra:** Sempre verificar o nome do charger no `nav_graph.yaml` e garantir que corresponde ao `config.yaml`. Ao renomear waypoints no traffic editor, atualizar o `config.yaml`.

---

## 2. reference_coordinates: compatibilizar pixels do mapa com coordenadas do ROS2

O Open-RMF trabalha com dois sistemas de coordenadas que precisam ser compatibilizados:

- **Pixels do mapa** (building.yaml): a imagem do mapa tem coordenadas em pixels, que o `building_map_generator` converte para metros usando as measurements (linhas de referência com distância conhecida). A transformação é: `x_rmf = px * escala`, `y_rmf = -py * escala` (Y invertido). No nosso caso a escala é uniforme: 0.006725 m/px.
- **Coordenadas do robô** (ROS2/Nav2): o robô opera no seu próprio frame, com origem e orientação definidas pelo mapa do Nav2.

O `reference_coordinates` no `config.yaml` é a ponte entre esses dois mundos. Ele define pares de pontos conhecidos em ambos os sistemas para que a biblioteca `nudged` calcule a transformação (rotação + escala uniforme + translação).

**O erro que cometemos:** os valores iniciais foram estimados manualmente e estavam errados. A posição do charger F no nav graph (em metros RMF) não correspondia à posição real do robô (em coordenadas ROS2). Resultado: o robô aparecia deslocado no dashboard e recebia destinos em posições físicas erradas.

**O procedimento correto:**
1. Identificar dois cantos opostos da sala (NW e SE) tanto no mapa quanto no frame do robô
2. No `building.yaml`, os cantos são os vértices das paredes (ex: vertex 5 = NW, vertex 7 = SE)
3. Converter os pixels para metros RMF: multiplicar pelo fator de escala do generator (derivado das measurements)
4. Parear com as coordenadas reais medidas no frame do robô
5. Usar esses 2 pontos (ou 4 para redundância) no `config.yaml`

**Verificação:** converter a posição init do robô (0, -0.48) pela transformação e confirmar que bate com a posição do waypoint charger no nav graph. Se não bater, os `reference_coordinates` estão errados.

**Regra:** Nunca estimar. Sempre derivar da transformação real do `building_map_generator` usando os vértices das paredes do `building.yaml`.

---

## 3. Theta +π: hack do dashboard que contamina o planner

O dashboard do Open-RMF subtrai π do yaw na renderização do robô (`robot-three.tsx: rotationZ = yaw - Math.PI`). Para compensar, adicionamos +π no `get_data()` do RobotClientAPI. O robô aparece na orientação correta no dashboard.

**O problema:** esse +π não afeta só a visualização. O planner DifferentialDrive do RMF TAMBÉM recebe esse theta e planeja todas as rotas achando que o robô olha para o lado oposto. Quando o destino volta pelo callback `navigate()`, o theta de chegada carrega esse offset de π.

**Exemplo real:** o robô está em F olhando para a direita (theta=0). O RMF recebe theta=π e pensa que o robô olha para a esquerda. O planner planeja: "preciso girar 180° antes de andar para a direita". Essa rotação é impossível para Dubin. E o theta de chegada enviado ao robô via VDA5050 está 180° errado, fazendo o Nav2 Dubin curvar para o lado oposto.

**A solução:** no `navigate()` do RobotClientAPI, ignorar completamente o theta que vem do RMF e calcular a direção de aproximação entre a posição atual e o destino:

```python
dest_theta = math.atan2(pose[1] - current_pos[1], pose[0] - current_pos[0])
```

Também aumentar `allowedDeviationTheta` de 0.5 rad (28°) para 1.57 rad (90°) no nó de destino VDA5050, dando liberdade ao Dubin planner do robô para escolher o ângulo de chegada viável.

**Regra:** Manter o +π no `get_data()` para o dashboard. No `navigate()`, sempre recalcular o theta com atan2. Nunca confiar no theta que o RMF envia de volta.

---

## 4. RMF usa planner DifferentialDrive — incompatível com Dubin

O RMF não tem planner Dubin. O planner DifferentialDrive faz duas coisas que são impossíveis para um robô Dubin:

1. **Rotação no eixo:** planeja o robô girando parado antes de andar. Ex: `t=0 yaw=180° → t=5.26 yaw=-9.5°` (giro de 190° parado em 5 segundos).
2. **Rota mais curta ignorando cinemática:** escolhe o caminho com menor distância/tempo, mesmo que exija manobras impossíveis. Ex: de A para D, roteia por F (passando por baixo) em vez de por B/C (passando por cima), porque é mais curto em distância — mas exige inversão de direção.

**Regra:** Contornar com lanes unidirecionais (lição 5) para eliminar rotas impossíveis, e com atan2 theta (lição 3) para eliminar rotações impossíveis.

---

## 5. Lanes unidirecionais para forçar sentido de circulação Dubin

Com lanes bidirecionais, o planner tem liberdade para escolher qualquer direção em qualquer lane. Para um robô Dubin, isso significa que ele pode ser enviado por um caminho que exige inversão de marcha ou curvas impossíveis.

A solução é tornar todas as lanes unidirecionais, formando um loop de sentido único (como uma rotatória). O robô sempre circula no mesmo sentido. Se todas as curvas do loop são para o mesmo lado, a cinemática Dubin é respeitada.

No nosso caso: loop anti-horário visto no dashboard (F→#0→A→...→B→...→D→...→E→...→#13→#0). Todas as curvas são para a ESQUERDA.

**Regra fundamental (validada em testes reais):** para qualquer robô que NÃO seja diferencial (Dubin, Ackermann, triciclo, rebocador), lanes bidirecionais são proibidas. Uma lane bidirecional implica que o robô pode percorrê-la nos dois sentidos, o que exige inversão de marcha — impossível sem ré. As tarefas só passaram a funcionar com sucesso quando TODAS as lanes foram convertidas para mão única.

**Mão dupla sem bidirecional:** se for necessário tráfego nos dois sentidos num corredor, criar DUAS lanes paralelas separadas (uma para cada sentido), cada uma unidirecional. Podem até pertencer a graphs diferentes (um por robô) para evitar conflito. Mas nunca uma única lane bidirecional.

**No traffic editor:** desmarcar "bidirectional" em todas as lanes. A ordem dos cliques ao criar a lane define o sentido (origem→destino). Se o sentido estiver errado, deletar a lane e recriar clicando primeiro na origem, depois no destino. Após salvar, regenerar o nav_graph e verificar que não há pares duplicados (lição 8).

---

## 6. Arredondar cantos com waypoints intermediários

Os cantos do loop original tinham ângulos de 90° — um waypoint no final de um trecho reto conectava diretamente ao início de outro trecho reto em direção perpendicular. Para um robô Dubin, isso é impossível: ele precisa de raio mínimo de curva e não pode fazer curvas fechadas.

A solução é adicionar 2-3 waypoints intermediários em cada canto, formando um arco suave. O robô recebe cada waypoint como destino sequencial, e o Nav2 Dubin planner traça curvas viáveis entre eles.

**Exemplo:** no canto superior-direito, em vez de um único vértice onde o robô iria de "subindo" para "indo à esquerda", colocamos 3 vértices em arco: um a 30°, outro a 60°, e o último a 90° da direção original. Cada segmento é uma curva suave que o Dubin consegue executar.

**Regra:** No traffic editor, adicionar 2-3 vértices em cada canto formando um arco. Conectar com lanes unidirecionais no sentido do loop. O raio do arco deve ser pelo menos o raio mínimo de curva do robô.

---

## 7. Traffic editor não funde vértices por sobreposição

Ao desenhar os arcos das curvas como segmentos separados dos trechos retos, o traffic editor cria vértices novos nos pontos de junção. Mesmo arrastando um vértice exatamente sobre outro, eles NÃO fundem — continuam sendo vértices separados com índices diferentes, sem lane entre eles.

O resultado no nav graph: o loop fica "quebrado" nos pontos de junção. O planner não consegue rotear através da quebra e o robô fica preso.

**Diagnóstico:** vértices duplicados têm coordenadas idênticas (ou quase) mas índices diferentes. Um aparece como "sem saída" e o outro como "sem entrada".

**Regra:** Para conectar dois segmentos: deletar um dos vértices duplicados e reconectar suas lanes no vértice que permaneceu. Após regenerar o nav_graph, verificar com script de validação (lição 9).

---

## 8. Nav graph unidirecional: representação no YAML

O `nav_graph.yaml` não tem parâmetro de direcionalidade nas lanes. A direcionalidade é implícita:

- Lane `[0, 1]` = só vai de 0→1
- Para bidirecional, o `building_map_generator` cria DUAS lanes: `[0,1]` e `[1,0]`
- Para unidirecional (`bidirectional: false` no `building.yaml`), cria apenas UMA

**Armadilha:** se você altera `bidirectional` no traffic editor mas não regenera o nav_graph, o nav_graph antigo (com pares duplicados) continua sendo usado. O fleet adapter usa o `nav_graph.yaml`, não o `building.yaml` diretamente.

**Regra:** Após qualquer mudança no traffic editor, SEMPRE regenerar o nav_graph com `building_map_generator`. Verificar o resultado: contar lanes — se há pares `[a,b]` e `[b,a]`, ainda é bidirecional.

---

## 9. Sempre verificar o nav_graph gerado com script de validação

O `building_map_generator` pode gerar nav_graphs com problemas sutis que não são visíveis no traffic editor:

- Vértices duplicados (mesma coordenada, índices diferentes)
- Loop aberto (dead ends onde deveria haver continuidade)
- Lanes no sentido errado
- Vértices sem saída ou sem entrada

**Script de verificação essencial:**
1. Construir grafo dirigido a partir das lanes
2. Traçar o loop completo a partir de F — deve voltar a F passando por todos os waypoints
3. Verificar que não há vértices "sem saída" (deveria ter lane saindo) ou "sem entrada" (deveria ter lane chegando)
4. Comparar coordenadas de todos os vértices para detectar duplicados (distância < 0.01m)

**Regra:** Rodar script de validação após CADA regeneração do nav_graph. Não confiar na visualização do traffic editor.

---

## 10. Procedimento correto para desenhar lanes no traffic editor

O traffic editor tem particularidades que geram problemas se não seguidas na ordem certa. O procedimento que funciona para um loop unidirecional com curvas Dubin:

**Passo 1 — Colocar todos os vértices primeiro**  
Antes de desenhar qualquer lane, posicionar TODOS os vértices do circuito: waypoints nomeados (A, B, D, E, F), intermediários dos trechos retos, e intermediários das curvas (2-3 por canto formando arco). Nomear os waypoints e marcar propriedades (`is_charger`, `is_parking_spot`).

**Passo 2 — Desenhar as lanes em sequência contínua**  
Desenhar as lanes uma a uma, seguindo o sentido do loop. Cada lane: clicar primeiro no vértice de ORIGEM, depois no de DESTINO. A ordem dos cliques define o sentido. Ir de um vértice ao próximo na sequência do loop, sem pular. Isso evita vértices duplicados porque você sempre clica em vértices que já existem.

**Passo 3 — Desmarcar bidirectional em cada lane**  
Imediatamente após criar cada lane, desmarcar "bidirectional" nas propriedades. Se esquecer, o `building_map_generator` vai gerar pares duplicados no nav_graph.

**Passo 4 — Fechar o loop**  
A última lane conecta o último vértice de volta ao primeiro, fechando o ciclo. Verificar visualmente que todas as setas apontam no mesmo sentido de circulação.

**O erro que cometemos:** desenhamos os trechos retos e as curvas como segmentos separados, cada um com seus próprios vértices. Resultado: vértices duplicados nos pontos de junção (mesmo lugar, índices diferentes, sem lane entre eles). O traffic editor não funde vértices por sobreposição, então o loop ficou quebrado em 5 pontos.

**A lição:** nunca desenhar segmentos separados e tentar juntar depois. Desenhar o circuito inteiro em sequência, vértice por vértice, lane por lane, do início ao fim e de volta ao início.
